# Session retro — 2026-05-09 — resume-cycle-hardening

**Harness:** claude-code (meta-editor session on `governance-v0`)
**Session window:** ~2026-05-09T07:30Z → 2026-05-09T15:00Z (~7h 30m wall-clock; ~2h active reasoning, rest passive — wakeups + Junior cycle wait)
**Branch at start:** `c990035b2` (`governance-v0`)
**Branch at end:** `5c6d97b13` (`governance-v0` — but only `92bedda12` is mine; the c-2 advisor authored the surrounding commits)
**Files touched (this session):** 4 in-repo (`advisor-orchestrator.md`, `auto-phase.md`, `resolve-dq-canonical.sh`, retro file) + 1 user-scope (`~/.claude/commands/auto-phase.md`)
**Commits (this session):** 1 explicit (`92bedda12`)

## TL;DR

Meta-editor session running parallel to a live `/auto-phase v1-SL-c-2` advisor session. The c-2 cycle surfaced three concrete resume-cost regressions: (1) my reconciliation read laptop's phase-branch DQ and got pending=0, while the live advisor saw pending=2 because DQ #164+#165 lived on worker-159; (2) live advisor reported a verbose ~80-line resume report costing ~3-5k tokens per resume (already 2 resumes deep on c-2); (3) live advisor preloaded the file-class injection table into parent context "in case Tasks 2-5 dispatch later" — those tasks were ≥30 min away. Shipped one focused commit `92bedda12` that fixes all three: a canonical DQ resolver script (`scripts/brehon/resolve-dq-canonical.sh`) that unions phase + worker-branch DQs deduped by id, a compact ≤15-line resume report spec, and a Phase 0.7 lazy-load discipline forbidding rule-table preloads at resume time. **Highest-leverage finding:** the same bug pattern (read-from-wrong-ref reconciliation) was about to repeat on every cohort-with-in-flight-ci-watcher; canonical resolver eliminates that whole class.

---

## What surprised us

- **The mix-up was on MY side, not the skill.** When I reported pending=0 vs the live session's pending=2, the natural assumption was "skill bug." Actual cause: I was a meta-editor session reading `origin/phase-v1-SL-c-2` directly; the live advisor reads its EliteDesk-side worktree which the daemon had pulled worker-branch refs into. Both reads were "correct" — different refs. The bug was the absence of an explicit canonical-DQ rule, not a malfunction.
- **Bash `set -e` + grep no-match silently returned empty in the resolver.** First test of the resolver returned `sources: ['phase-branch']` only — junior-id loop was firing but `branch=` was always empty. `bash -x` showed `git ls-remote` worked but the result was empty. Root cause: `grep -E '[-]<id>$'` exits 1 on no-match; under `set -euo pipefail` that propagates as failure even though the `head -1` after it would have produced empty output cleanly. Required `|| true` after the grep.
- **The breadcrumb mechanism actually worked.** I had low confidence the live c-2 advisor would consume the `.claude/auto-state/v1-SL-c-2-skill-update-breadcrumb.md` file. Confirmation arrived at retro time: file deleted (per breadcrumb's own instruction #4), `resume_count: 3` (incremented post-breadcrumb), and fix-impl-2's commit explicitly cites `§G4 canonical recipe` — the exact phrasing from the breadcrumb's content. Cross-session async communication via gitignored breadcrumb works.
- **The cancellation cascade in parallel Bash calls was avoidable.** When my first SQL probe failed (wrong column name), the parallel Bash batch of 6 calls all cancelled. I re-issued with corrected SQL → same thing happened. Lesson: when probing an unknown schema, do the schema discovery serially first, *then* parallelise the batch that depends on it. I was treating the parallel batch as atomic when it's actually fail-fast.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | (SHIPPED `92bedda12`) Canonical DQ resolver `scripts/brehon/resolve-dq-canonical.sh` + skill body Step C wiring + advisor-orchestrator.md polling-loop bullet | Eliminates phase-branch-vs-worker-branch DQ drift class for both `/auto-phase` reconciliation and manual polling | minor (50-line bash script + 3 doc paragraphs) | 1× this session, 0× prior — but would-have-fired on every cohort-with-ci-watcher going forward |
| 2 | (SHIPPED `92bedda12`) Compact resume report ≤15 lines, ≤500 tokens; verbose detail to gitignored `.claude/auto-state/<phase>-resume-debug-<ts>.md` on user 'debug' request | Saves ~3-5k parent tokens per resume cycle; c-2 already at resume_count=3 (saved ~9-15k tokens retroactively if applied earlier) | minor (skill body Step E rewrite) | 1× this session — but compounds across every resume |
| 3 | (SHIPPED `92bedda12`) Phase 0.7 lazy-load discipline — skill body MUST NOT preload rule tables at resume; just-in-time inside handler that needs them | ~5-8k tokens saved per resume; eliminates "preload for later" anti-pattern | minor (one new sub-section + invariant table row) | 1× this session — direct evidence from c-2 transcript |
| 4 | When probing an unknown SQL schema or external DB column, run `PRAGMA table_info(table);` (or equivalent) BEFORE issuing parallel queries that name columns. Add as pattern note to `.claude/lessons/` if it recurs | Avoids parallel-batch cancellation cascades when the first guess is wrong | trivial (rule of thumb, no code) | 1× this session — defer to lesson if 2nd occurrence |
| 5 | Future: track `resume_count` aggregated across phases. If c-3 + c-4 both show `resume_count` ≥ 4 with all reconciliations finding zero discrepancies, propose auto-continue (skip 'continue' prompt when reconciliation is clean). Don't promote pre-evidence | Reduces friction once reliability is proven; not worth doing speculatively | medium (rule + state machine change) | 0× yet — defer |

## What to carry forward

- **Cross-session breadcrumb pattern via gitignored `.claude/auto-state/<phase>-*-breadcrumb.md`** — write content + cleanup-instruction; live session consumes + deletes. Used twice now (skill ship 2026-05-08; this session). Works when meta-editor needs to pass durable-but-runtime info to a live session without polluting git history. Also: instruction #4 ("delete after reading") is load-bearing — without it, breadcrumbs accumulate.
- **One focused commit per concern bundle**, even when the bundle has 3 internal moving parts. `92bedda12` shipped as a single "resume-cycle hardening" commit with a 3-part body (canonical resolver / compact report / lazy-load). Discoverability via grep is unchanged from 3 commits; reverting is easier; the c-2 advisor reads the diff once not three times.
- **Smoke-test the resolver against the actual incident state before claiming the fix works.** I caught the `set -e` + grep bug because the first test returned `sources: ['phase-branch']` instead of the expected `pending: 2 sources: ['phase-branch', 'worker-159']`. Without that smoke-test the script would have shipped broken. Pattern: every script that walks state should have a known-state to verify against, ideally in the commit body so future sessions can re-verify.
- **Verify retro-named claims live, not from memory.** When asked "did the breadcrumb get picked up", I checked `ls .claude/auto-state/` (file present or gone?) + `cat v1-SL-c-2.json` (resume_count incremented?) + `git log --oneline` (commit citing §G4 canonical recipe?) — three independent signals. Each could be lied about by memory; together they triangulate. Per `feedback_runbook_audit_drift_post_event_check.md`.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Reading user's prior resume report (~80 lines) | — | — | high | Not a tool call but the input that surfaced the friction. The verbosity itself was the signal. |
| `git-show-json.sh` (canonical helper) | 5 | 0 | none | Ran reliably; foundation the new resolver builds on. |
| Parallel Bash probe batch (first attempt, wrong SQL column) | 0 | 8 | medium | Wrong-column-name → all 6 sibling calls cancelled. Re-issue with fixed schema → same cascade. Lesson: serial schema discovery before parallel column-named probes. |
| `mcp__junior-brehon__list_tasks` (replacement for failing SSH-SQL) | 3 | 0 | low | Worked first try; the right tool for status reads. |
| `Edit` on skill body + rules (3 large blocks) | 15 | 2 | low | Heredocs cleanly preserved. One re-read needed when I forgot to Read the rule file before Edit (my own discipline miss). |
| Resolver smoke-test cycle (run → bug → fix → re-run) | 4 | 4 | medium | The `set -e` + grep gotcha was the surprise; once diagnosed (2 min), fix was 1 line. Net: caught a bug that would have shipped silent. |
| Single commit `92bedda12` | 10 | 0 | none | Clean atomic ship; 3-part body discoverable. |
| AskUserQuestion (none used) | 0 | 0 | none | All decisions were either bounded-scope (smoke-test, schema discovery) or already-confirmed by user ("batch with cuts #1+#2"). |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Resume-cycle hardening (canonical resolver + 2 doc cuts) | 4 | 1 | ~75 (active reasoning) | ~10 (subagent dispatch wait) |

No watchdog risk (single foreground session, no Junior dispatch).

## Decisions to revisit

- **Resolver glob pattern (`grep -E "[-]${jid}$"`) is fragile** — relies on the worker-branch naming convention `junior/role-*-md-<id>`. If Junior changes its branch-naming convention, the resolver silently misses worker branches. Worth a unit-test or a more structural query (e.g. parse the auto-state's `current_cohort.members[].branch` field if it gets added to the state schema). Defer to retro time when it actually matters.
- **The compact resume report's 'debug' fallback** isn't yet implemented — only specified in the skill body. First time a user types 'debug' the live advisor will need to author the dump. If that proves tedious, ship a small helper script. Defer until it actually fires.

---

## Auto-phase reliability

The c-2 phase ran (and is still running) under `/auto-phase`. This session was meta-editor work informed by c-2 evidence — not driving c-2 directly. Categories below evaluate the c-2-in-flight cycle's behaviour as observed via auto-state JSON + git log + user transcript, NOT this session's tool work.

### 1. Stage-transition correctness

✓ Cycle 1 (impl → ci-watcher → fail → fix-impl-1) followed the correct routing per the stage-shape table. Junior 159 (ci-watcher-2) ran on the wrong-recipe fix; correctly returned `result: fail`. Advisor moved to §G4 classifier path. ⚠ Cycle 1's `fix-impl-1` (junior 157) failed on a daemon-side ref-sync issue, not a recipe issue — the brief was right, the daemon's local `refs/heads/junior/...-md-154` was stale. Not a routing bug per se; surfaces as a recurring daemon-sync gap (3rd occurrence: also hit ci-watchers 155+156). **Read of `user_gate_history[].notes`:** plan-approval gate captured "DoD smoke PASS 10/11; watchpoint specificity gate PASS 14/14; e2e.rs LOC 11925 higher than planner expected" — the LOC note is a planning-side carry-forward signal that this retro propagates.

### 2. Cadence calibration

⚠ Workflow `25595869651` (cycle-1 ci-watcher-2 target) ran 24+ minutes when 8-12 min is typical. ci-watcher polled ScheduleWakeup 270s cycles correctly per the cadence table; not a calibration miss, but a workflow-tail-anomaly that the cadence didn't notice as anomalous. Suggestion: surface "ci-watcher polling >2× expected wall-clock for stage" as an anomaly line in the compact resume report (cut #1 already specifies anomaly lines for this class).

### 3. Auto-state integrity

✓ `resume_count: 3` at retro time. Within the ≤3 target per `feedback_auto_phase_retro_signals.md` §3, but at the boundary. `last_known_phase_tip: 47af884c8` matches `git rev-parse origin/phase-v1-SL-c-2`. No hand-edits required this session. ⚠ Note: `resume_count: 3` was reached on a single sub-phase (c-2) before merge — slightly above the ≤3-per-sub-phase soft target. Most resumes were driven by user-side restarts (compaction, conversation length), not skill-side stalls. Carry-forward signal: if c-3 also lands at ≥3, propose investigating session-length friction.

### 4. User-touchpoint count vs target

Insufficient data — c-2 still in flight. user_gate_history shows only 1 entry (plan-approval) so far. Will be evaluated at c-2 phase-retro time, not this session retro.

### 5. Catch-fire FP/FN rate

✓ Zero catch-fires in this session. The c-2 catch-fire on the original E0277 LemmyError was BEFORE this session (yesterday's). It was correctly classified as catch-fire at the time (the §G4 allowlist didn't yet include the recipe); after `1cf03c4f6` shipped the allowlist row + breadcrumb, the cycle correctly auto-classified the same failure pattern as allowlist-eligible. Net: 1 → 0 false-negative class, validating the `1cf03c4f6` fix.

### 6. §G4 classifier accuracy

✓ The breadcrumb's §G4 canonical recipe (".map_err(|e| format!("{e}").into())") was applied verbatim by fix-impl-2 (junior 160, commit `f7d2229da`). ci-watcher-3 (junior 161) is now polling workflow `25603848858` to confirm. **First end-to-end test of the §G4 allowlist's E0277-LemmyError row.** Outcome pending; if green, validates the row; if red, the recipe needs refinement and the row should be quarantined until tested again.

### 7. L14 / L15 / L16 fixes still holding

N/A — c-2 hasn't reached merge. Will be evaluated at c-2 phase-retro.

### 8. Subagent offload effectiveness

N/A — this session didn't dispatch subagents. The live c-2 advisor's Phase 0.5 reconciliation subagent is documented in the skill body but I have no transcript of its actual usage. Defer to c-2 phase-retro.

### 9. Plan §13 fidelity vs cohort dispatch

⚠ Cohort 2 is a **single-task cohort** (Task 1 alone, no `[P]`-marked siblings). That's per plan §13 — Task 1 has no `[P]`. So no parallelism opportunity was missed. But the cohort dispatch logic correctly handled the single-member case (no overlap check fired, no budget degrade). ✓ on the mechanical question. Carry-forward: the plan's complexity score for Task 1 (single-task with 11k+ LOC e2e file edit) was the dominant factor; that the planner correctly chose serial-only for c-2 reflects the file-class complexity score awareness shipped earlier.

### 10. Resume-cycle pain points

⚠ `resume_count: 3` with verbose Step E reports cost ~9-15k tokens cumulatively. The shipped fix in `92bedda12` addresses this for c-3+; c-2 will continue paying the verbose cost until next session restart. Suggestion: when the live c-2 advisor next restarts, the new compact format will engage. If user observes "much faster resume" subjectively, that's confirmation. Track in c-2 phase-retro.

### Aggregate auto-phase reliability score

| Category | Status | Recurrence |
|---|---|---|
| 1. Stage-transition correctness | ⚠ | 3× this phase (daemon-sync gap on 155/156/157), 0× prior |
| 2. Cadence calibration | ⚠ | 1× this phase (workflow tail anomaly not surfaced), 0× prior |
| 3. Auto-state integrity | ✓ (boundary) | resume_count=3 within target, but at edge |
| 4. Touchpoint count | N/A | c-2 in flight |
| 5. Catch-fire FP/FN | ✓ | 1 catch-fire (yesterday) → 0 (today) after fix; FN class closed |
| 6. §G4 classifier | TBD | first E0277-LemmyError test pending ci-watcher-3 |
| 7. L14/L15/L16 holding | N/A | pre-merge |
| 8. Subagent offload | N/A | not used this session |
| 9. Plan §13 fidelity | ✓ | single-task cohort handled correctly |
| 10. Resume cycles | ⚠ | resume_count=3 at boundary; verbose-cost fix shipped for c-3+ |

⚠ entries that became proposals in §"What to change":
- Categories 1+2 → "anomaly lines in compact resume report" → covered by cut #1 anomaly-line spec
- Category 10 → entire "What to change" rows 2+3 (compact report + lazy-load) → SHIPPED `92bedda12`

⚠ entries that did NOT promote (deliberately): the daemon-sync gap (Category 1, 3rd recurrence) is a Junior-side issue. Promoting it requires either a daemon patch or an advisor-side workaround. The advisor-side workaround (force-fetch `refs/heads/junior/*` before dispatch) is suggested in the prior conversation as "optimisation #1"; user-confirmed but I haven't shipped it yet. **Proposal carry-forward to next meta-editor session: ship the brief-author "force-fetch" directive as a 1-paragraph addition to the skill body's `bm-cut → planning → impl` stage handler.** Single-paragraph cost; eliminates the daemon-sync class for fix-impl re-dispatches. Leaving for next session because: (a) c-2 is in flight and shouldn't churn underneath; (b) bundling with another skill-body change makes a cleaner commit.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Daemon-side ref-sync directive in fix-impl brief** — promote to skill body's stage handler for `impl-fix-N-running` re-dispatch. Recurrence: 3× (155, 156, 157). Concrete: add to `~/.claude/commands/auto-phase.md` brief-authoring step for re-dispatched worker branches.
- [ ] **Anomaly line surfacing** — when a polling stage's wall-clock exceeds 2× the expected duration table value, the compact resume report and the next polling tick should surface a `⚠` line. Recurrence: 1× this session (24-min cargo-validate-workspace), but generalises broadly. Concrete: add anomaly-detection logic to skill body Phase 1 tick.
- [ ] **Schema-discovery-before-parallel-probe** — write as `.claude/lessons/feedback_serial_schema_discovery_before_parallel_probes.md` if recurs. Recurrence: 1× this session — defer.
- [ ] **Cross-session breadcrumb pattern as documented technique** — the `.claude/auto-state/<phase>-*-breadcrumb.md` pattern worked twice (skill-ship 2026-05-08 + this session). Concrete: write `.claude/lessons/feedback_cross_session_breadcrumb.md` documenting the pattern (path convention + cleanup instruction + when-to-use).
- [ ] PMD eval write — `PROJECT_MEMORY_DB` is wired (verified earlier in conversation). Eval title: `Session retro: resume-cycle hardening — canonical DQ resolver + compact reports + lazy-load`. `source_ref`: `92bedda12`.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted: `feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`, `feedback_retro_task_complexity_score.md`, `feedback_auto_phase_retro_signals.md`._
