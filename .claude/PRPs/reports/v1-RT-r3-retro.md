# Phase retro — v1-RT-r3 — multi-source participation_consistency emitters + flag-bad-faith admin endpoint

**Sub-phase:** v1-RT-r3
**Phase branch:** `phase-v1-RT-r3` (deleted at merge)
**PR:** #155
**Merge SHA:** `5ebd8ae23131…` into `governance-v0`
**Merged:** 2026-05-28T17:37:24Z
**Phase wall-clock:** 2026-05-24 → 2026-05-28 (~4 days)
**Trunk position post-merge:** `7b9046a3d` (BM bookkeeping)

## TL;DR

v1-RT-r3 shipped the four PRD §5.3 reputation-event sources (activity-cron, dormancy-cron, vote-outcome, evidence-cited) plus the `flag-bad-faith` admin endpoint that produces the negative arm of the evidence-quality source. 4 plan §13 tasks delivered + 3 fix-impl cycles. **Total 28 files changed, +3269/-1167.** Three failure classes recurred mid-phase — all advisor-side defects, not Junior-side: (1) **Junior #479 `error_max_turns`** on fix-impl-2 brief (same class as Task 4 #474–#477); recovered by user-authorised advisor-authorship carve-out (#87285bbfe). (2) **Junior #480 stale-base read** — worker forked from stale daemon-local ref pre-brief, filed a false-blocker DQ; recovered by daemon-local ref reset + redispatch (#481 succeeded in 3m45s). (3) **Mode A brief-on-trunk procedure gap** — bm-merge brief target is `governance-v0` but lane is on phase branch; resolved via daemon-temp-worktree SSH path (commit `621bc7115`). Headline acceptance gates all pass: 4 sources + flag-bad-faith endpoint shipped; cargo check + clippy clean at every cycle; CR triage flips approve with 0 open fix-in-pr; PR #155 merged with task-per-commit history preserved (`--merge` not `--squash`).

## Four-role retro signals (per `feedback_four_role_retro_signals.md`)

### Planner role
- **Plan quality:** strong. `.claude/PRPs/plans/v1-RT-r3.plan.md` enumerated 4 sources + flag-bad-faith with explicit dedupe-key formulas, hit "Headline acceptance" 1:1 with PR #155 diff. No mid-phase plan revisions.
- **§13 tasks:** Task 0 (pre-flight) + Task 1 (participation_cron) + Task 2 (vote-outcome + evidence-cited) + Task 3 (flag-bad-faith) + Task 4 (e2e suite). Tasks 1+2+3 dispatched as cohort-2 `[P]`; all PASSED validate-pending-laptop in one cycle (`ddb439553`).
- **Watchpoint specificity:** §4 watchpoints cited specific files (participation_cron.rs, scheduled_tasks.rs) — DoD smoke test + watchpoint-specificity gate passed at plan approval.

### Impl-task role
- **Cohort-2 (Tasks 1+2+3 [P]):** clean. Cohort YAML overlap-check + `requires:` dependency check passed; all three workers completed; fix-impl-1 (clippy::too_many_arguments) auto-allowlisted + cleared in one cycle. Sonnet performed exactly as the model envelope predicts on small-file targets.
- **Task 4 (e2e suite, NOT [P]):** cycled #474–#477 in `error_max_turns` (4 failed dispatches against the 8945-line `crates/server/tests/e2e.rs`); cycle-count §5.3 HARD REFUSAL fired at 3 cycles → user-authorised (B) advisor-side authorship; landed cleanly at `87285bbfe` (10/10 fixtures PASS).
- **fix-impl-2 (cr-1 DQ boundary + cr-3 iso-week guards):** Junior #479 same class as Task 4 — 38 min / $8.30 / 151 turns `error_max_turns`. Brief was 246 lines × 3 file edits including e2e.rs; over the per-session-retro envelope. Recovered by advisor carve-out (`90cd0183d`).
- **fix-impl-3 (cp-1/cp-2/cp-3 + cr-12):** Junior #480 stale-base false-blocker; Junior #481 retry from origin tip succeeded in 3m 45s. cp-targets are 500-line cron files — well inside the Sonnet envelope. Validate-pending-laptop PASS (`73a1fe54d`).

### BM-task role
- All BM verbs mechanical-reliable. bm-cut (`dc9781a74`), bm-pr (#155 open from gov-side worktree workaround per fed-in-b pattern), 3× bm-poll-cr (poll-2 + poll-3), 2× bm-triage (initial + post-fix-impl-3 promotion), bm-merge (#483, 2m 03s execute-side). Phase 5.5 post-condition verify caught no false-success.
- **bm-merge ran without incident.** No retries; no `git push -f` attempts; trunk fast-forwarded cleanly; runlog COMPLETE entry POST-merge per L14 fix; remote branch deleted via `--delete-branch`.

### Advisor role
- **Cycle-count §5.3 HARD REFUSAL fired correctly.** Surfaced Task 4 cycle to user at the third repeat of `(error_max_turns, e2e.rs)`; user chose (B) corrective replan; advisor-authorship carve-out shipped without further cycles. The cycle-count meta-rule paid for itself.
- **Stale-base recovery worked first try.** Junior #480 forked from stale daemon-local ref; advisor caught the false-blocker DQ, falsified its premise (brief IS at origin tip), proposed 3 recovery options via AskUserQuestion, user picked daemon-local reset + redispatch, #481 succeeded.
- **Surface-first ritual + falsifiable-hypothesis discipline both fired in real-time.** Stale-base diagnosis used `feedback_falsifiable_hypothesis_before_structural_fix.md` recipe: verified brief AT origin tip BEFORE proposing structural fix; this avoided treating the DQ's RCA ("brief missing") as fact.
- **Mode A brief-on-trunk procedure gap surfaced — needs codification.** The bm-merge brief target is `governance-v0` (BM worker reads from trunk per `advisor-orchestrator.md` §2.1), but the lane session is locked to `phase-v1-RT-r3` per `multi-lane-worktree.md` Hard refusal #1. Procedure isn't covered in §"Brief location and trunk→phase sync" (which addresses impl-task briefs only). Resolved ad-hoc by daemon-temp-worktree SSH path. Codify as a new mode-A subcase OR add a `bm-merge-brief-author.md` recipe. // 1× this phase; record as note, watch for 2× before promoting to rule.

## Lessons promoted this phase

- **Daemon-local trunk stale on multi-lane** (`feedback_daemon_local_trunk_stale_multi_lane.md`) — already in MEMORY.md but reconfirmed: worker forks from daemon-LOCAL ref, not `origin/<branch>`; ff via `git fetch origin <branch>:<branch>` is lane-safe. RT-r3 Junior #480 was the 4th confirmed instance. No new authorship; existing lesson stands.
- **Junior cancel pre-flight tar** (already in `advisor-orchestrator.md` §5.6) — did NOT fire this phase; #474–#477 cancellations all happened mid-`error_max_turns` (worker wrote no code) so tar wasn't load-bearing. Pattern stays in place for cohort cancellations.
- **Cohort dispatch shared-`.git/index.lock` hazard** (`feedback_cohort_shared_git_index_contention.md`) — fired earlier in v1-RT-r3 (cohort-2 was 3-wide → degraded-to-serial check triggered). Per-task worktrees ran serially; all 3 PASSED. Lesson held.
- **Cycle-count meta-rule >= 3 → catch-fire** (`advisor-orchestrator.md` §5.3) — fired correctly on Task 4 (#474–#477). User-authorised carve-out path worked. Confirms 2× threshold; lesson stays.

## What surprised us

- **Cohort-2 (Tasks 1+2+3) cleared in ONE Junior cycle.** Three small-file targets (participation_cron.rs new, submit_jury_vote.rs touch, admin_emergency_remove.rs touch) ran in parallel, all PASSED their fixtures, fix-impl-1 was a single auto-allowlisted `clippy::too_many_arguments` silence. Predicted: 1–2 fix-impl cycles based on prior RT-r1 + RT-r2 rates. Actual: 1 cycle. This is the canonical positive-case for `[P]` cohort dispatch with proper file-disjointness.
- **Task 4 (e2e suite) cycled 4× before user-authorised carve-out.** 4 fix-impl dispatches all `error_max_turns` on the same 8945-line `e2e.rs` target. Brief was 350+ lines × 2 e2e Edits. The §3.2 gate-3 risk-surfacing change (still pending promotion per session-retro-2026-05-26 #3, 1× threshold) would have flagged this AT dispatch time. **Promote that now → 2× threshold met (Task 4 + fix-impl-2).**
- **Junior #480 stale-base false-blocker recovery was clean.** The worker filed a DQ blocker citing "brief missing"; advisor falsified in ~3 tool calls (brief IS at origin tip + worker forked from `5921bd60d` not `94857fa0c`); proposed 3 recovery options with AskUserQuestion; user picked option (a) daemon-local reset; #481 succeeded. End-to-end recovery: ~3 min advisor time, ~4 min Junior time.
- **bm-merge brief had to land on TRUNK, not phase branch.** The lane is locked to phase-v1-RT-r3; trunk authorship requires daemon-temp-worktree SSH. This isn't in any current rule. Authored ad-hoc; needs a §"Trunk-author from lane (bm-merge brief case)" subsection in `multi-lane-worktree.md`.
- **Commit subject escape sequence preserved literally in git.** SSH-via-bash heredoc with `\xe2\x80\x94` (em-dash escape) stored the escape *as text* rather than the rendered character. Cosmetic only (brief content is binary-correct), but worth noting for future SSH-author paths: prefer single-quoted heredoc OR Edit-then-commit-via-file rather than inline `-m '...— ...'` over SSH.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Promote session-retro-2026-05-26 #3 (gate-3 failure-class risk surfacing) to a shipped change.** Add to `.claude/rules/advisor-orchestrator.md` §3.2 gate 3: when fix-impl brief targets a known-fragile pattern (e2e.rs ≥2 edits, multi-helper refactor, JSON syntax fix in a 4000+-line DQ), the AskUserQuestion at user-gate-3 MUST include "this brief matches the X failure class signature — N% retry rate observed" in one option description. | User authorises with full risk picture; no silent dispatch into known traps. Prevents Task-4-cycle-recurrence class. | minor (one sentence per gate prompt + a small lookup table) | **2× threshold now met** — Task 4 cycle (#474–#477) + fix-impl-2 (#479). Both saw e2e.rs ≥2 edits + no upfront risk-surfacing. Both burned >30 min before catch-fire. |
| 2 | **Codify Mode A trunk-authorship from a phase lane** in `.claude/rules/multi-lane-worktree.md` §"Brief location and trunk→phase sync" — add a "Mode A trunk-author" subcase: when a lane session must commit to `governance-v0` (e.g. bm-merge brief, lesson edit), use the daemon-temp-worktree SSH pattern (`git worktree add /tmp/brehon-gov-tmp governance-v0 → commit → push → worktree remove`). Cite the v1-RT-r3 bm-merge brief commit `621bc7115` as the canonical example. | Lane sessions get a documented procedure; no more ad-hoc invention. Eliminates "should I switch this lane to governance-v0?" Hard-refusal-#1 ambiguity. | minor (rule prose only) | 1× this phase; record as note, watch for 2× before stronger codification. |
| 3 | **DQ schema-v3 cosmetic improvement:** when authoring an SSH-via-bash commit subject containing em-dashes, prefer single-quoted heredoc `-m "$(cat <<'EOF' ... EOF)"` over inline `-m '... — ...'`. Add an example to `feedback_windows_backslash_path_dq_via_write_fragment.md` (already covers Windows backslash class; same lesson applies to SSH-utf8). | Cosmetic-only; prevents `\xe2\x80\x94` text in commit subjects. | trivial | 1× this phase; cosmetic only; record as note, no rule change yet. |

## Per-task complexity scores

Per `feedback_retro_task_complexity_score.md` shape: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`. **Aggregated below; individual rows from runlog.**

| Task | Files | Impl commits | Runtime (min) | Max log silence | Notes |
|---|---|---|---|---|---|
| Task 0 (pre-flight audit) | 1 | 1 | ~7 | n/a | clean |
| Task 1 (participation_cron) | 4 | 1 | ~9 | n/a | clean validate-pending-laptop PASS |
| Task 2 (vote-outcome + evidence-cited) | 2 | 1 | ~6 | n/a | clean |
| Task 3 (flag-bad-faith endpoint) | 2 | 1 | ~6 | n/a | clean |
| fix-impl-1 (clippy::too_many_arguments silence) | 1 | 1 | ~5 | n/a | auto-allowlist |
| Task 4 e2e suite (Junior cycles) | n/a | 0 | 4 × ~30 min cycles = 120 min wasted | log silence n/a (max_turns hit) | cycle-count §5.3 HARD REFUSAL fired |
| Task 4 e2e suite (advisor carve-out) | 2 | 3 | ~120 (advisor wall-clock) | n/a | 10/10 fixtures PASS at `87285bbfe` |
| fix-impl-2 (cr-1 + cr-3) (Junior #479) | n/a | 0 | ~38 (error_max_turns) | n/a | brief verbosity × e2e.rs fragility |
| fix-impl-2 (advisor carve-out) | 2 | 1 | ~25 (advisor) | n/a | landed at `90cd0183d` |
| fix-impl-3 (cp-1/2/3 + cr-12) (Junior #480) | n/a | 0 | ~5 (stale-base false-blocker) | n/a | daemon-local trunk stale class |
| fix-impl-3 (Junior #481, retry) | 4 | 3 | ~4 (Junior) + ~5 cargo gates (advisor laptop) | n/a | clean PASS |

**Aggregated:** ~3269 lines added / ~1167 lines removed across 28 files. 4 plan tasks + 3 fix-impl cycles → 7 dispatches of which 4 cycled (3× Task 4 max_turns recoveries to carve-out + 1× fix-impl-2 max_turns to carve-out + 1× fix-impl-3 stale-base to redispatch). **Net Junior success rate: 5/9 dispatches first-try (Tasks 0–3 cohort-2 + fix-impl-1 + fix-impl-3-retry); 4 require advisor intervention (Task 4 + fix-impl-2 carve-outs + fix-impl-3 stale-base).**

## Carry-forwards from PR #155 CR triage (issues filed)

| # | Severity | Summary | URL |
|---|---|---|---|
| 156 | major | process-env mutation safety doc-comment overstates guarantees on multi-thread tokio runtime | https://github.com/barrie-cork/lemmy/issues/156 |
| 157 | minor | DQ entries with `resolved_at` earlier than `timestamp` (clock-skew in handover-block fields) | https://github.com/barrie-cork/lemmy/issues/157 |
| 158 | nit | extract shared `emit_reputation_event` helper | https://github.com/barrie-cork/lemmy/issues/158 |
| 159 | minor | `boot_context` env vars not wrapped in `EnvVarGuard` (leak risk on early `?`) | https://github.com/barrie-cork/lemmy/issues/159 |
| 160 | minor | `LEMMY_DATABASE_URL` set without `EnvVarGuard` | https://github.com/barrie-cork/lemmy/issues/160 |

## DQ entries authored this phase

17 DQ commits across phase branch. Mix of validate-pending-laptop (impl-task → advisor-laptop mutation cycle, 4 pairs) + advisor-side carve-out logs (DQ -031, -032, -033) + the stale-base false-blocker (Junior #480 DQ `f0d0ee0d45af-001`) which the advisor resolved by recovery rather than answering. All resolved before merge.

## Headline acceptance criteria — verification

| Story | Status | Evidence |
|---|---|---|
| Source 1 (activity cron) emits `+1 participation_consistency` per (user, community) per ISO week | ✓ | `participation_cron.rs::run_activity_batch` (cohort-2 commit `b9876b7d8`) + fixtures at `e2e.rs::v1_rt_r3_fixtures::participation_cron_*` (10/10 PASS in `87285bbfe`) |
| Source 2 (dormancy cron) emits `-2 participation_consistency` per (dormant user, community) per ISO week | ✓ | `participation_cron.rs::run_dormancy_batch` + fixtures |
| Source 3 (post-decision vote-outcome) emits `+1` per majority-aligned juror; minority emit 0 | ✓ | `submit_jury_vote.rs::emit_vote_outcome_events` (commit `996765cae`) + no-penalty-for-dissent assertion in fixtures |
| Source 4a (evidence-cited heuristic) emits `+1 reporting_accuracy` to reporter on long-rationale + ≥1 evidence row | ✓ | `submit_jury_vote.rs::emit_evidence_quality_events` + fixtures |
| Source 4b (`POST /flag-bad-faith`) emits `-1 reporting_accuracy`; 403/400/200 cap+status arms | ✓ | `admin_emergency_remove.rs::flag_bad_faith` (commit `e544ae26e`) |
| Dedupe-key idempotency under repeat-tick | ✓ | `feca72af0` clamp+warn ensures interval ≥1; fixtures assert idempotent on second tick |

All 6 headline rows ✓. Plan §16a §13 task list 1:1 with shipped diff.

## Open follow-ups (not blocking trunk)

- **Promote session-retro-2026-05-26 #3** (gate-3 failure-class risk surfacing) — see Change #1 above; 2× threshold now met.
- **Codify Mode A trunk-authorship** — see Change #2; record as 1× note, watch.
- **5 CR carry-forward issues** (#156–#160) — none gate v1-RT-r4 or any other v1 lane; address opportunistically in next RT lane or v1-housekeeping-r1.

## Sign-off

- Plan §13 tasks shipped: ✓
- Headline acceptance criteria pass: ✓
- CR triage: 0 open fix-in-pr, 6 done, 5 carry-forward (filed), 7 wont-fix; recommendation `approve` ✓
- PR #155 merged via `--merge` (task-per-commit history preserved): ✓
- Advisor post-condition verification (Phase 5.5): ✓
- Retro authored: ✓

**Awaiting user sign-off (gate 6) → `/brehon-phase-transition`.**
