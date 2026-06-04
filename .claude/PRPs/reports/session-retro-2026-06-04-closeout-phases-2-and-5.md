# Session retro — 2026-06-04 — closeout-phases-2-and-5

**Harness:** claude-code
**Session window:** 2026-06-04 ~14:45 → ~15:20 UTC (~35 min active)
**Branch at start:** `f42f19ce2` (`phase-v1-closeout`)
**Branch at end:** `cfcb49175` (`phase-v1-closeout`)
**Files touched:** 4 tracked (+ 2 user-scope PMD/MEMORY files, + 1 untracked workflow script)
**Commits:** 2 (auto: 0, explicit: 2)

## TL;DR

A `/auto-phase v1-closeout` invocation that *couldn't run* (wrong branch + no
four-role phase to drive while M1 owns the daemon) became a productive manual
close-out session: shipped Phase 2 (doc-drift reconcile, via a `Workflow`-tool
scan) and Phase 5 (wasmtime deferral decision), and verified the DQ archive was
already complete. The dominant, recurrent finding: **the close-out plan's stated
facts were stale in three independent places**, and a verify-against-reality pass
caught all three before they anchored a wrong action — Phase 2's RT-r2 fix was
"repoint" not "annotate" (a dedicated retro file had since been authored), and
Phase 5's two load-bearing premises ("13 aarch64-only criticals", "watch open PR
#847") were both wrong (12 alerts with real x86_64/arch-neutral exposure; PR #847
closed/stale/wrong-direction). Top change proposal: treat **plan-stated external
facts (alert counts, upstream PR/release state, file existence) as hypotheses with
a mandatory pre-action verify**, the same discipline `feedback_runbook_audit_drift_post_event_check`
already prescribes for runbooks — this session is its 3rd+ recurrence and it earns
a stronger preventative.

---

## What surprised us

- **`/auto-phase` was the wrong tool for the request, and the request's intent was a
  reasonable-but-wrong shorthand.** The user's prior-session decision "next session
  runs `/auto-phase`" read as "drive the close-out under proper machinery" — but
  `/auto-phase` hard-refuses off `governance-v0` AND only orchestrates the four-role
  Junior pipeline (the 🟩 lane), which is exactly what's blocked while M1 owns the
  daemon. The *actual* next phase (Phase 2 doc-drift) is a 🟦 `Workflow`-lane task.
  Surfacing the mismatch (not forcing `/auto-phase` to refuse, not silently
  substituting) was the right call — but it's surprising that a recorded user
  decision pointed at a structurally-inapplicable tool.

- **The plan was stale in THREE separate, independent places** — and all three were
  load-bearing for the action:
  1. Phase 2 RT-r2: plan said "annotate if a dedicated retro is authored"; the
     dedicated `v1-RT-r2-retro.md` *had* been authored → fix was repoint, not annotate.
  2. Phase 5: "13 aarch64-only criticals" → actually 12 alerts, only 2 criticals
     aarch64-only; ~10 medium/low with real x86_64/arch-neutral exposure.
  3. Phase 5: "watch extism PR #847 (open since 2026-04-07)" → #847 is CLOSED
     (2025-05-19), wrong-direction (wasi/wiggle 31, older), and extism already
     shipped its wasmtime-41 bump in v1.21.0 (our pin) via a *different* PR (#897).
  Three stale facts in one ~90-line plan section is a higher drift rate than expected
  for a plan authored *this same session-day*.

- **The DQ "archive" task was already done by a prior session — and the script's
  v3-awareness made that safe to discover.** The plan recorded "545 KB / 273
  resolved (over both triggers)"; live state was 311 KB / 166 resolved, and the
  dry-run showed **0 eligible entries** (all 30 remaining legacy int-ids are cited;
  all 136 v3 composite-ids are structurally never-eligible). The `dq-archive.sh`
  filtering v3 string-ids out of the integer-cutoff comparison meant the live M1
  entries (all composite-id) were never at risk — a genuinely well-designed guard.

- **The `/schedule` watch the plan asked for would have silently lapsed.** Both
  `CronCreate` (session-scoped) and `/schedule` recurring jobs auto-expire at 7
  days, but the wasmtime-≥42 trigger horizon is months. The plan's "set a `/schedule`
  watch" instruction predated awareness of that expiry. A durable MEMORY.md/PMD
  watch entry was the correct substitute.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Plan-stated external facts are hypotheses — verify before acting.** Add a one-line gate to the close-out plan's remaining phases (and the lesson `feedback_runbook_audit_drift_post_event_check.md`): before acting on any plan-asserted *external* fact — Dependabot alert count, upstream PR/release number+state, a "file X exists / doesn't exist" claim — re-verify it live (`gh api`, `WebFetch`, `ls`) in the same step. | Stops stale-plan facts from anchoring wrong actions (3 caught this session; the cost of NOT catching the RT-r2 one would have been a wrong "annotate" edit, and the PR-#847 one a dead cron watch). | minor (doc + lesson edit) | 3× this session; ≥2× prior (`feedback_runbook_audit_drift_post_event_check`, `feedback_verify_automated_reviewer_claims_against_compiler`) → **promote** |
| 2 | **Update the close-out plan's stale facts in place** so the next session doesn't re-inherit them: Phase 2 RT-r2 line → "repoint (dedicated retro exists)"; Phase 5 → "12 alerts (not 13); 2 criticals aarch64-only, rest medium/low incl. 1 x86_64; watch extism RELEASES not PR #847 (closed/stale); extism 1.21.0 already on wasmtime-41". | Plan becomes self-consistent with verified reality; removes the drift that change #1 catches reactively. | minor (plan edit) | 1× this session (the drift itself) |
| 3 | **Record the `/schedule` 7-day-expiry footgun as a lesson.** New `feedback_schedule_cron_expiry_vs_watch_horizon.md`: when a watch trigger's horizon exceeds ~7 days, use a durable MEMORY.md/PMD watch entry (re-read each session) NOT a `/schedule` cron (auto-expires at 7 days → false "we're watching"). Links to `feedback_thin_wakeup_prompts_verify_live_state`. | Future "set a watch on X" requests pick the mechanism that matches the horizon; avoids silent-lapse false confidence. | minor (lesson) | 1× this session + 1× prior (the deferred-spec + Telegram-bridge MEMORY.md watches already use the durable pattern implicitly) → borderline; surface as candidate |
| 4 | **`.claude/workflows/` tracking convention is undecided** — left untracked this session. Decide: track reusable close-out workflow scripts (Phase 8 governance-TODO sweep is another 🟦 lane) or gitignore the dir as runtime-only. | Removes the per-session "track or not" judgment; either commits the reusable scan harness or stops it showing in `git status`. | minor (1 decision + `.gitignore` line or `git add`) | 1× this session (noise only) |

## What to carry forward

- **Ground-truth-gather inline BEFORE fanning out a workflow.** The Phase 2 workflow
  was scoped to *verify + fix-draft* because the advisor gathered the 3 drift inputs
  inline first — which immediately surfaced that `schema.rs` wasn't at the assumed
  path and a dedicated RT-r2 retro existed. Discovery-in-parent, verification-in-agents
  kept the workflow tight (3 claims, 5 agents, ~3.5 min) and caught the path/file
  surprises before they reached the agents. Repeat for Phase 8.

- **`Workflow` tool with `pipeline()` + adversarial-verify stage is an excellent fit
  for doc-drift / audit-classification work** — and it's the M1-isolation-safe lane
  (laptop subagents, no daemon). Each claim's verify fired as soon as its scan
  finished (no barrier). The `agentType: 'Explore'` read-only agents + per-claim
  `SCAN_SCHEMA`/`VERDICT_SCHEMA` produced directly-usable structured output. This is
  the close-out's primary engine while M1 holds the daemon.

- **Dry-run before any irreversible DQ/archive mutation, always.** The `--dry-run`
  on `dq-archive.sh` proved the work was already complete and that forcing the
  trigger would mean dismantling deliberate teaching citations (`#37` attribution
  example, `#50` collision). Zero mutation, full certainty. Same discipline applied
  to the v3-append helper (author fragment via Write, helper injects id) — no
  hand-edited JSON, no Windows backslash-path class.

- **Surface tool/intent mismatches as an `AskUserQuestion`, don't auto-substitute.**
  When `/auto-phase` couldn't run, offering 4 concrete paths (Workflow / drive-hybrid
  / force-auto-phase / status-only) let the user steer rather than guessing which
  reading of "run `/auto-phase`" they meant.

- **Re-check the M1 gate each session** (`git log origin/phase-m1-b ^origin/governance-v0`)
  — +29 commits this session (M1 mid-Task-6); NO-ELITEDESK still binding. Mechanical,
  cheap, gates which phases are even reachable.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`. This session ran NO
four-role Junior work (M1 owns the daemon) — the operating roles were **advisor**
(parent session) and **workflow-subagents** (laptop `Agent` dispatch). Scored on
that basis.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/auto-phase` invocation | 0 | ~3 | medium | Refused (wrong branch + no four-role phase under NO-ELITEDESK). The ~3 min "wasted" was actually well-spent: reading the rule + reconciling state surfaced the tool/intent mismatch cleanly. Not a true loss. |
| `Workflow` tool (phase2-doc-drift, 5 agents) | ~40 | 0 | low | Verified 3 drift claims + adversarially checked each fix in ~3.5 min / ~234k subagent tokens. Manual doc-vs-code verification of 3 claims (incl. locating real schema files + reading 3 retro files) would have been ~40 min of serial parent work + heavy context. High signal-to-noise. |
| Inline ground-truth gather (pre-workflow) | ~10 | 0 | medium | Caught `schema.rs`-not-at-path + dedicated-RT-r2-retro-exists BEFORE the workflow ran → sharpened agent prompts. The "medium" surprise = the RT-r2 fix flipping from annotate→repoint. |
| `WebFetch` ×3 (extism PR #847 + releases + crates.io) | ~15 | 0 | high | Caught the two biggest Phase 5 plan-staleness items (PR #847 dead; extism already on wasmtime-41). Without it, would have set a dead-cron watch + documented a wrong "aarch64-only" rationale. Highest-surprise, highest-value probe of the session. |
| `gh api` Dependabot alert enumeration | ~5 | 0 | high | Revealed "13 aarch64-only criticals" was wrong (12 alerts; x86_64/arch-neutral exposure present). Reframed the security rationale to be honest. |
| `dq-archive.sh --dry-run` | ~20 | 0 | medium | Proved the archive was already complete (0 eligible) → avoided a no-op (or worse, a citation-dismantling) mutation. The v3-string-id filtering protected live M1 entries. |
| `dq-v3-append-fragment.sh` | ~5 | 0 | none | Clean composite-id append; no hand-edited JSON, no backslash-path class. Worked first try. |
| `AskUserQuestion` ×4 (path / Phase2-apply / Phase5-decision / watch-mechanism) | ~5 | 0 | none | Each was a genuine user-owned decision (tool choice, apply-gate, ADR-012 risk, watch-mechanism). No false-positive gates. |

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. The canonical metric
(`files/commits/runtime/log-silence`) targets **Junior impl-task watchdog envelope**
— **N/A this session** (zero Junior tasks; M1 owns the daemon). The `Workflow`-run
analog is recorded instead for trend-tracking of the close-out's primary engine:

| Task | Agents | Subagent tokens | Wall-clock (min) | Notes |
|---|---:|---:|---:|---|
| Phase 2 doc-drift Workflow (`wdg2fylk0`) | 5 | ~234k | ~3.5 | 3 claims × (scan + verify), pipeline (no barrier). Within the plan's "small cost-gauge first run" intent. No watchdog concept (laptop subagents, not Junior). |

No flags (the watchdog thresholds — >55min runtime, >40min silence, >8 files — are
Junior-specific and don't apply to laptop `Workflow` subagents).

## Decisions to revisit

- **`.claude/workflows/` tracking** (What-to-change #4) — decide before Phase 8 adds
  another workflow script.
- **Should the close-out plan's "Session progress & state" living-block be the
  authoritative status, given `/auto-phase` doesn't drive this lane?** The plan
  already says so; worth confirming the next session reads the plan's living-state
  (not `/auto-phase` auto-state, which will never exist for this lane).
- **Dependabot alert dismissal is a pending user action** (12 wasmtime alerts + the
  T3 tar-dispute). Not a retro item — a tracked obligation surfaced to the user.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 (verify plan-stated external facts before acting): **augment** existing
  `.claude/lessons/feedback_runbook_audit_drift_post_event_check.md` with the
  "external facts = alert counts / upstream PR-release state / file existence" axis
  (3× this session; the lesson + `feedback_verify_automated_reviewer_claims_against_compiler`
  already establish the pattern → meets ≥2 threshold). Cross-harness lesson.
- [ ] Change #2 (update close-out plan stale facts in place): direct edit to
  `.claude/PRPs/plans/v1-closeout.plan.md` Phase 2 + Phase 5 sections. Not a lesson —
  a one-time plan correction.
- [ ] Change #3 (`/schedule` 7-day-expiry vs watch-horizon): new
  `.claude/lessons/feedback_schedule_cron_expiry_vs_watch_horizon.md` (borderline —
  1× strong this session + the durable-watch pattern already in MEMORY.md implicitly).
- [ ] PMD eval write: requires the HTTP PMD reachable; `memory_write` for the
  change-#1 takeaway (`Session retro: plan-stated external facts are hypotheses —
  verify alert counts / upstream PR state / file existence before acting`).

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase reliability section omitted —
`/auto-phase` was invoked but refused at the branch gate; it never advanced the
state machine, so no auto-state artifact exists and the 10-category section is
dormant per Step 0.5._
