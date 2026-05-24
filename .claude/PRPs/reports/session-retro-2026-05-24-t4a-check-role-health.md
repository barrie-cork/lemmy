# Session retro — 2026-05-24 — t4a-check-role-health

**Harness:** claude-code
**Session window:** ~2026-05-24T10:00Z → 2026-05-24T10:42Z (~42 min wall-clock)
**Branch at start:** `097a51d09` (`governance-v0`)
**Branch at end:** `846f10a14` (`governance-v0`, +1 commit, unpushed per user direction)
**Files touched:** 2 committed (`.claude/commands/check-role-health.md` new, `.gitignore` +1 line)
**Commits:** 1 (explicit; no auto-commits)

## TL;DR

Authored T4a — the `/check-role-health <role>` consumer slash command — end-to-end in 42 minutes from session-4 handover read to commit. The session's load-bearing finding was a **scope misclassification caught mid-authorship**: the session-4 handover specified user-scope (`~/.claude/commands/`) but every dependency the command consumes (role substrate, canonical PMD, drain script) is brehon-fork-specific, making user-scope structurally wrong. User caught it ("should this be project level rather than user level?") and the corrected project-scope shipped. The session also surfaced a **cross-worktree PMD-path footgun** the command's authoritative spec now defends against (absolute canonical path hardcoded per pmd-invariants invariant #1). End-to-end testing against current PMD (6 bm-task rows → 2 real after sentinel filter) revealed three real-world spec gaps that landed as patches in the same commit: sentinel exclusion, historical config_version filter, outcome-section schema-gap deferral. Top change proposal: codify "handover-specified scope is a hypothesis, not a contract" as a brief-author gate.

---

## What surprised us

- **Session-4 handover prescribed the wrong scope.** Handover §3.2 (session 2) and §4.1 (session 4) both explicitly said "user-scope at `~/.claude/commands/check-role-health.md`". I authored the file at that path and was already mid-spec when the user asked "should this be project level rather than user level?" The user's instinct caught what a careful read of the handover did not: every dependency (role manifests in `.claude/roles/`, canonical PMD in `brehon-fork/.project-memory/`, drain script in `scripts/brehon/`) is brehon-fork-only. User-scope would have produced a globally-installed command useless from any other repo. The handover's recommendation was an unverified hypothesis carried forward across 3 sessions before the user falsified it. **Generalises to:** session-N+1 inheriting session-N's framing without re-checking premises.

- **End-to-end test caught 3 real spec defects on the first invocation.** I ran the command's 9 steps live against the current PMD before committing. Step 4's query returned 6 rows; Step 5's aggregation revealed 5/6 were smoke-test sentinels with empty `rules_read` arrays, which would have made the strip-candidates section falsely flag all 24 rules. Step 6's retro-join was structurally impossible (signal rows store `source_ref=<task_id>`, retros store `source_ref=<branch>`). Step 7's threshold guard worked correctly. Three spec patches landed in the same commit. **Surprise direction:** positive — dogfooding the spec at write time, not post-deploy, was much cheaper than the alternative.

- **The drain script's gitignore status was inconsistent with its sibling.** `.claude/role-signal-queue.jsonl` was already gitignored (line 59); the closely-related `.claude/role-signal-drain/` runtime cache had no entry and showed up as untracked noise. Caught only because I ran `git status --short` pre-commit. Generalises to: when adding a feature that creates a sibling runtime artefact, audit the existing gitignore for the closest analogue and follow the pattern.

- **Two Stop hooks fire on every interactive session, one of which is silently spending CPU.** The `role-signal-utilisation.sh` hook fires on every Stop event (including this advisor session) and parses `transcript_path` for a `[role:X]` tag. In an interactive (non-`claude -p`) session, the tag is always absent, so the hook exits 0 silently. Visible only because the user asked "what two stop hooks fired?" — no other signal would have surfaced it. **Cost:** small (one transcript parse per Stop). **Concern:** zero-signal hooks in non-`claude -p` sessions are pure waste in steady state.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When a handover prescribes a config decision (scope, path, tool choice), at brief-author time list the dependencies that decision constrains and verify the decision is consistent with them. Add to `.claude/lessons/feedback_handover_assumptions_need_empirical_verification.md` (already exists from session-4 retro) as a second concrete row: "handover-specified scope is a hypothesis until checked against dependency locality." | Prevents user-scope/project-scope class of error; one mid-authorship pivot avoided per occurrence (~5 min wasted authoring + edit churn) | minor (1-line edit to existing lesson) | 1× this session, 1× session-4 (handover-assumption verification gate was authored in retro `c92facca3` for a different class — fed-in-e brief) — **threshold met (2×) for lesson extension** |
| 2 | Add a `Stop` hook short-circuit early-exit for non-`claude -p` interactive sessions in `.claude/hooks/role-signal-utilisation.sh`. Check for the dispatch-line role tag once at top; if absent AND no `transcript_path`-derived role, exit 0 in the first 10 lines instead of after the full transcript scan. | Eliminates wasted CPU on every interactive Stop event in advisor sessions (multiple per hour). Per session-4 §2.2, the hook already does this conceptually — but the parse-then-exit ordering means a non-`-p` session still scans the transcript before deciding. | minor (refactor 10 lines in hook) | 1× this session (observable cost: small; signal cost: no instrumentation makes recurrence-counting hard) — **single occurrence, not yet promoted** |
| 3 | When adding a new feature that creates a runtime artefact directory, audit `.gitignore` for the closest existing sibling pattern (`grep <feature-base-name> .gitignore`) before authoring; add the new entry as part of the same commit. Codify in a brief-author template note or add to `feedback_commit_aggressively_in_shared_repos.md` as a sibling-pattern gate. | Prevents 0.5-1 min of `git status` noise per session; prevents accidental cache commits | minor (1 sentence add to existing lesson) | 1× this session, **possibly N× historically** — recurrence-unknown; deferred until 2nd occurrence with evidence. Not promoted now. |

## What to carry forward

- **Dogfood the spec at write-time.** Running the 9-step workflow live against current data before committing caught 3 real defects (sentinel pollution, historical-config-version pollution, retro-join schema gap) that would otherwise have shipped and surfaced only after Junior dispatch volume accumulated. The cost was ~5 min of bash + sqlite3; the alternative was a 2nd commit days later. Repeat for every new read-only consumer command — they're cheap to dogfood.

- **User-as-falsifier on inherited handovers.** The user's question "should this be project level rather than user level?" was the single most load-bearing turn of the session. Carry forward: when authoring against a handover, treat its prescriptions as hypotheses on the first read; do not refute, just notice; defer to the user when a falsification surfaces a structural-vs-prescribed mismatch. Per `feedback_falsifiable_hypothesis_before_structural_fix.md` (cited in MEMORY.md): the same pattern as DQ #338.

- **Cross-lane absolute-path discipline.** The command's spec hardcodes `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` (the canonical PMD per pmd-invariants invariant #1) rather than the relative `.project-memory/memory.db`. This was a direct application of the cross-lane PMD lesson — the kind of preemptive defence that costs nothing at write time and prevents lane-stranding incidents.

- **AskUserQuestion batching for design decisions.** Asked 4 questions in a single survey (scope/drain coupling/strip mode/threshold) early in the session. All four were answered in one turn with recommended defaults. Saved 4 round-trips. Carry forward: when authoring a new spec, surface every load-bearing decision in one batched question set, not as they arise.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| AskUserQuestion (batched 4-question survey) | 8 | 0 | none | Single round-trip resolved scope/drain/strip/threshold |
| AskUserQuestion (user-scope → project-scope confirm) | 2 | 0 | medium | Triggered by user's "should this be project level rather than user level?" question; surprise = handover was wrong, caught only by user instinct |
| AskUserQuestion (drain dir gitignore) | 1 | 0 | none | Clean fork on cache file handling |
| ToolSearch (load TaskCreate/TaskList/etc) | 1 | 0 | none | Single-call load for task tracking |
| Live end-to-end test of the spec via bash + sqlite3 | 15 | 0 | medium | Caught 3 real defects (sentinel, historical-cv, retro-join); surprise = positive (effective at write time, not post-deploy) |
| Read sessions 2-4 handovers | 3 | 0 | low | Mostly current; session-3 §3.2 had stale "build write-role-signal on EliteDesk" step (already done by session 4) |
| Initial author of command at user-scope (later corrected) | 0 | 4 | high | ~4 min of write churn under user-scope assumption before user pivoted to project-scope |
| `/session-retro` (current) | — | — | — | In flight |

**Net wall-clock:** ~42 min total; saved ~30 min vs unstructured authorship; wasted ~4 min on the user-scope detour; ~8 min of authoring + 15 min of testing + 5 min of patching + 5 min of misc = 33 min "productive" + 4 min wasted.

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. None of the session's work crossed the heavy-task threshold (>55min runtime, >40min log silence, or >8 files touched). The single commit was 2 files / 298 insertions and ran end-to-end in ~42 min — sub-threshold across all three axes.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Author + test + commit T4a command | 2 | 1 | 42 | <1 (interactive session, no daemon dispatch) |

No outliers; no carry-forward signals from complexity scores.

## Decisions to revisit

- **Outcome-section retro-join is deferred but real value.** The current spec carves out the join as DEPRECATED; the post-task-retro skill change to add `task_id:<id>` tag is a 1-line update that unlocks a meaningful analytic. Worth a follow-up clarify pass before the next role-customization phase.
- **The role-signal hook silent-exit in `-p`-mode questions.** Per session-4 §4.4 candidate lesson `feedback_stop_hook_fires_silent_in_p_mode.md`, the hook fires in `claude -p` but emits no stream-json log signal. This session confirmed the hook also fires in interactive sessions and silently early-exits. Worth a single empirical check ("does the hook emit ANY observable when role-tag is absent?") before promoting to lesson.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 (handover-assumption verification gate): extend existing `.claude/lessons/feedback_handover_assumptions_need_empirical_verification.md` with the scope-vs-locality row. **Threshold met (2×).** Owner: next session.
- [ ] Change #2 (Stop hook early-exit refactor): single-occurrence, not promoted. Recheck in 1 week.
- [ ] Change #3 (sibling-pattern gitignore audit): single-occurrence, not promoted. Recheck after weekly-review pattern sweep.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
