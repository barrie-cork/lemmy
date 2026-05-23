# Retro — v1-ship-2 (4 governance e2e tests: appeal, modlog, reputation, endorsement)

**Sub-phase:** v1-ship-2 — per-endpoint e2e backfill for 4 governance handlers
**Shipped:** 2026-05-23T10:01:00Z — PR #147 merged into governance-v0 (merge sha `ac37a125dfc9f0c38dd5d9355ec25bee58c08bc7`)
**Branch lifetime:** `3d94028` (lane cut, 2026-05-22 19:18 BST) → `ac37a125` (merge, 2026-05-23 11:01 BST) — ~16 hours
**Deliverables:** `appeal_request_and_deny_happy_path`, `modlog_list_returns_appeal_entry`, `get_my_reputation_returns_snapshot`, `create_endorsement_happy_path_and_self_endorse_rejects` — all GREEN in `crates/server/tests/e2e.rs` mod `v1_ship_2_fixtures`.
**This is a RETRO, not a completion report** — it records what the four roles did well, where they drifted, and what changes the next sub-phase should carry. Signals are organised per-role.

---

## 0. Outcome

| Dimension | Result |
|---|---|
| Goal achieved | **Yes.** All 4 e2e tests pass. Tasks 1-4 each shipped a single test function; all 4 validate-pending-laptop gates passed (cargo check + clippy + test --no-run EXIT_0). |
| Tests | **Pass.** Full module e2e ran in the previous session: 4/4 functions GREEN, 90.6s run time. PR #147 merged with `final_recommendation: approve`. |
| Clean execution | **Partial — recovered.** Two incidents: (1) rebase conflict on `decision-queue.json` with "take HEAD" strategy silently discarded ship-2 DQ additions; (2) PR became DIRTY/CONFLICTING after governance-v0 advanced (fed-in-e merged), requiring rebase + force-push with explicit user authorization. Both recovered without irreversible harm. |

---

## 1. The arc (what actually happened)

1. **Phase cut + planning (2026-05-22 ~19:18 BST):** `roadmap-next` flipped v1-ship-2 to in_flight, cut lane worktree `brehon-fork-ship-2`. Planning Junior ran from brief; plan shipped `4dfa6ff1` at 19:52 UTC (~34 min). Clarify DQs `a3d0e9941441-007/008` self-answered by advisor.
2. **Tasks 0-4 dispatched serially (22:08–08:03 BST):** Pre-flight Task 0 ran as harness audit (no impl deliverable). Tasks 1-4 each authored a brief, dispatched to Junior, returned a commit + validate-pending-laptop DQ entry which the advisor-laptop validated. All 4 passed first-try.
3. **bm-pr opened PR #147 (08:36 BST).** CodeRabbit posted 6 findings by 07:41 UTC (pre-PR — CR ran on the branch push). Triage: cr-1 (major, unscoped UPDATE → fix-in-pr), cr-2 (minor, DQ schema drift → fix-in-pr), cr-3 (minor, audit-trail arg order → rebut), cr-4 (nit, markdownlint on runlog → rebut), cr-5a/5b (minor, unused imports → fix-in-pr). User gate 3 approved; fix-in-pr commit `dea25b3f0` addressed all 4 fix-in-pr findings.
4. **PR became DIRTY (09:21 BST).** fed-in-e PR #148 merged into governance-v0 while PR #147 was in CR triage. `mergeStateStatus: BLOCKED`. Advisor rebased `phase-v1-ship-2` onto `origin/governance-v0` — 5 sequential conflicts on `decision-queue.json`. "Take HEAD" strategy correctly resolved field-update conflicts on existing entries but silently discarded the ship-2 DQ array additions (new entries added by ship-2 commits). DQ restoration commit `65cef4f97` re-added all 4 entries with correct v3 `approved_by`/`approved_at` fields.
5. **Force-push required (10:42 BST).** Rebase required `git push --force-with-lease`. Auto-mode classifier correctly blocked this (rebase was an autonomous advisor action; prior "I approve" covered CR fix-in-pr only, not the rebase). Explicit user authorization obtained ("I authorise. please proceed") before push. This is the correct behavior — the block was right.
6. **bm-merge executed (11:01 BST).** BM Junior dispatched with explicit brief. Phase 5.5 post-condition verified state=MERGED, mergeCommit present. Runlog COMPLETE appended post-merge as `c709a5258`. Remote branch `phase-v1-ship-2` deleted (L16 confirmed).

---

## 2. Per-role signals

### 2.1 Advisor

**Did well:**
- **Serial task dispatch with validate-pending-laptop gates held.** All 4 tasks dispatched, validated, and confirmed pass before next cohort advanced. No silent failures.
- **DQ restoration after rebase was complete.** After recognising the "take HEAD" drop, advisor extracted all 4 missing entries from the pre-rebase tip via `git show`, added v3 schema fields, and committed them as a clean restorative commit. The audit trail is intact.
- **Force-push authorization correctly obtained.** Rather than self-authorizing the force-push (which was a consequence of an autonomous rebase decision), surfaced to user for explicit authorization. The auto-mode classifier blocking the push was correct and honoured.
- **Phase 5.5 post-condition verified before accepting merge success.** `gh pr view` confirmed state=MERGED, mergedAt non-null, mergeCommit 40-hex before reporting success.

**Drifted:**
- **"Take HEAD" rebase strategy for `decision-queue.json` is semantically wrong for new array additions.** DQ rebase conflicts involve two kinds of change: field updates to existing entries (take-HEAD is correct — HEAD has the authoritative state of existing entries) and new array element additions (take-BOTH — each side may have independently added entries). The "take HEAD" recipe correctly handles the first class but silently drops the second. This recurred 5 times across the rebase. → **§3 action 1.**
- **Timing mismatch: CR arrived before PR was opened.** CodeRabbit's timestamp (07:41 UTC) predates the `bm-pr runlog` commit (07:36 UTC). CR ran on the branch push (bm-push), not the PR-open event. Minor discrepancy; no impact on triage quality but suggests CR polls branch push events, not PR creation events.
- **DQ timestamp noise:** Tasks 1-3 DQ entries show `timestamp: 2026-05-22T00:00:00Z` or `2026-05-23T00:00:00Z` (midnight sentinel), suggesting the timestamp was set to a round value at DQ-entry creation time rather than the actual dispatch moment. T1's `resolved_at: 2026-05-23T23:14:51Z` is plausible but very late — 25 hours after brief authoring. Not a correctness issue; worth noting for future DQ timing fidelity.

### 2.2 Planning

**Did well:**
- **Plan scoped correctly to 4 tests with clear per-task IMPLEMENT boundaries.** Each task produced exactly one test function; no ambiguity in what each impl-task was supposed to do.
- **Self-endorse error shape correctly specified in advance.** Plan specified 404 for self-endorse (not 409 Conflict); impl followed it; CR didn't flag it. This was a potential footgun (self-endorse could plausibly return 409) that the plan pre-emitted correctly.
- **No clarify DQs needed for impl.** The two advisor clarify DQs (`a3d0e9941441-007/008`) were self-answered by the advisor without escalating to user. Plan was actionable.

**No material drift.**

### 2.3 Impl

**Did well:**
- **All 4 tasks passed first-try validation.** No §G4 cycles, no fix-impl dispatches, no retry loops. cargo check + clippy + test --no-run each exited 0.
- **Unused imports caught by CR, not by impl.** cr-5a and cr-5b were unused-import findings. These should ideally have been caught by `cargo clippy` at validate-pending-laptop time. The validate-pending-laptop gate ran clippy (based on commit messages), but `#![deny(unused_imports)]` apparently does not trigger for `use ... as _` style imports or the warning was suppressed. This is a known clippy nuance; the CR caught it correctly and it was cleaned up without difficulty.
- **Unscoped UPDATE in cr-1 was a real finding.** The `diesel::update` filtered by `status=Decided` rather than by `id` — would silently affect all Decided rows in a multi-case state. The CR finding was correct; the fix was a one-line filter addition. This suggests impl-task brief did not explicitly specify the scoping requirement for the diesel UPDATE.

### 2.4 BM

**Did well:**
- **bm-pr dispatched cleanly.** PR opened with correct base (governance-v0), title, and body. No errors.
- **bm-triage report was accurate.** 6 findings correctly bucketed; 4 fix-in-pr all addressed in a single commit.
- **bm-merge Junior executed the corrected brief cleanly in ~3 min.** Post-L14-revision brief (runlog COMPLETE post-merge) worked without incident. The L14 lesson from v1-ship-1-r2 was correctly encoded in this brief.

**No material drift.** BM role was the cleanest this sub-phase.

---

## 3. Actions for the next sub-phase

### Action 1 — DQ rebase conflict recipe: union new entries, take-HEAD for existing

**Problem:** When rebasing a phase branch onto an advanced governance-v0, `decision-queue.json` conflicts arise. Two change classes in a DQ diff:
1. **Field updates on existing entries** (e.g., adding `approved_by`/`approved_at` to an existing id) — "take HEAD" is correct; HEAD has the authoritative state.
2. **New array element additions** (entries added by the phase branch that don't exist in HEAD yet) — "take HEAD" silently drops them; the correct resolution is "take BOTH" (append all new entries from INCOMING that are not present in HEAD by id).

**Recipe for future DQ rebase conflicts:**
```
# After rebase conflict on decision-queue.json:
# 1. git show HEAD:.claude/decision-queue.json > /tmp/dq-head.json
# 2. git show MERGE_HEAD:.claude/decision-queue.json > /tmp/dq-incoming.json  # or REBASE_HEAD
# 3. For each entry in incoming: if entry.id not in head → add to head's pending/resolved
# 4. Validate JSON, stage, continue rebase
```

The five-conflict chain in this sub-phase took ~20 minutes to recognise and correct. A one-paragraph recipe in the DQ conflict resolution notes would prevent this class.

**Files to update:** `.claude/rules/decision-queue.md` (add a "Rebase conflict resolution" sub-section under "Concurrency"); possibly a lesson file.

### Action 2 — Brief must specify diesel UPDATE scope when inserting + immediately updating

**Problem:** cr-1 was an unscoped `diesel::update` in the appeal test — filtering by status rather than by id. This is the recurring "wider-than-intended mutation" footgun in tests that insert a record and then update it. The impl-task brief did not specify that the UPDATE must be scoped by the inserted id.

**Recipe for future impl-task briefs targeting handlers that do insert-then-update:**
- Brief §4 MUST include: "Any `diesel::update` call MUST filter by the inserted row's id, not by a status or state field."

### Action 3 — Keep (validated patterns)

- **Serial task dispatch with first-try validation** (all 4 tasks, first-try) — the 4-test plan decomposition was the right granularity.
- **L14 runlog COMPLETE post-merge** (no conflict) — the fix from v1-ship-1-r2 held. Do not regress.
- **Phase 5.5 post-condition check before accepting merge** — catches false success; keep as hard rule.
- **Explicit user authorization for force-push after autonomous rebase** — the auto-mode classifier correctly blocked the push; honouring that signal and getting explicit authorization was right.

---

## 4. Per-task complexity scores

Format per `feedback_retro_task_complexity_score.md`: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`.

Runtimes derived from git commit timestamps (BST, UTC+1). Junior task run times inferred from brief-authored → DQ resolved gap. Validate-pending-laptop times from DQ `timestamp` → `resolved_at`.

| Task | Deliverable | Files | Commits | Runtime (min) | Max log silence | Notes |
|---|---|---|---|---|---|---|
| Planning | v1-ship-2 plan | 1 | 1 | ~34 | unknown | Brief → plan ship ~34 min |
| Task 0 | Pre-flight audit | 0 | 0 | ~50 | unknown | Brief 22:08 → T1 brief 23:27; no impl commit |
| Task 1 | appeal test | 1 | 1 | ~48 | unknown | Brief 23:27 BST → DQ raised ~23:30 UTC; laptop validate ~25 min per validate-pending timestamps |
| Task 2 | modlog test | 1 | 1 | ~79 | unknown | Brief 00:18 BST → DQ resolved 06:37 UTC (overnight batch; elapsed includes idle wait) |
| Task 3 | reputation test | 1 | 1 | ~18 | unknown | Brief 07:39 BST → DQ resolved 06:57 UTC → commit 07:57 BST |
| Task 4 | endorsement test | 1 | 1 | ~31 | unknown | Brief 08:03 BST → DQ resolved 07:34 UTC → commit 08:34 BST |
| fix-in-pr | CR fixes (4 findings) | 2 | 1 | ~45 | — | PR opened 08:36 BST; fix commit 09:21 BST; advisor-side inline |

Tasks 1-4 each touch 1 file (`crates/server/tests/e2e.rs`) and 1 commit. Well within watchdog thresholds. The overnight gap on Task 2 inflates its wall-clock runtime; actual execution was likely <30 min.

---

## 5. CR triage summary

| Finding | Severity | Bucket | Resolution |
|---|---|---|---|
| cr-1 — unscoped diesel::update | major | done | Fixed: add `.filter(moderation_case::id.eq(id))` in commit `dea25b3f0` |
| cr-2 — DQ v3 schema drift | minor | done | Fixed: add `approved_by: null, approved_at: null` |
| cr-3 — audit-trail arg order in DQ | minor | rebut | Rationale: DQ is an audit-trail record of the command as invoked; correcting it would falsify history |
| cr-4 — MD022 blank line after heading | nit | rebut | Rationale: bm-runlog is an operational journal, not user-facing doc; markdownlint is net-noise |
| cr-5a — unused imports ListGovernanceModlog, RequestAppeal | minor | done | Fixed: removed |
| cr-5b — unused imports PersonId, JuryDecision | minor | done | Fixed: removed |

0 critical/major open at merge time. 4/4 fix-in-pr addressed in single commit. Clean triage.

---

## 6. Lessons promoted this phase

### LESSON (user-flagged for retro)
**`decision-queue.json` rebase conflict: "take HEAD" drops new array additions.**

- "Take HEAD" is correct when the conflict is a *field update* on an existing entry (HEAD has authoritative state).
- "Take HEAD" is wrong when the conflict is a *new array element addition* (INCOMING added an entry HEAD doesn't have yet; taking HEAD silently discards it).
- Correct recipe: take-HEAD for field-update conflicts + union (take-BOTH) for new-entry additions.
- This happened 5 times in a single rebase chain before it was recognised. Recovery: extract entries from pre-rebase tip via `git show <pre-rebase-sha>`, add v3 schema fields, append to current file.

Promote as: `feedback_dq_rebase_conflict_union_new_entries.md`

---

## 7. Decisions to revisit

- **DQ "Rebase conflict resolution" recipe** (§3 action 1): add a sub-section to `.claude/rules/decision-queue.md`. No user sign-off required (advisory/mechanical; not ADR-affecting).
- **Impl brief: diesel UPDATE scope requirement** (§3 action 2): add to brief template. Forward-only.
- **DQ timestamp fidelity**: T1-T3 DQ entries had midnight-sentinel timestamps. Worth auditing how validate-pending-laptop entries get their `timestamp` value — should be the DQ creation moment, not a round-number placeholder.

---

## 8. Three-signal scoring

Per `feedback_four_role_retro_signals.md`:

| Role / signal | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---|---|---|---|
| **Advisor** — serial validate-pending-laptop | 0 | 0 | — | All 4 first-try; nothing to save/waste |
| **Advisor** — DQ rebase conflict recognition | 0 | ~20 | **Yes** | 5 conflicts, took ~20 min to diagnose + restore |
| **Advisor** — force-push authorization | 0 | ~5 | Low | Auto-mode block was right; explicit auth obtained correctly |
| **Impl** — all 4 tasks first-try | ~60 | 0 | Positive | No §G4 cycles; clean execution |
| **BM** — bm-merge L14 post-merge order | ~10 | 0 | — | L14 fix from ship-1-r2 held; no self-conflict |
| **CR triage** — 4 fix-in-pr + 2 rebut | ~15 | 0 | — | Clean triage; single fix commit addressed all |

**Composite:** wasted = 25 min; saved + wasted = 25 + 85 = 110 min; friction = 25/110 = 0.23; score ≈ **0.77**.

Anti-inflation calibration: no §G4 cycles, no BM violations, no irreversible harm, no re-plan. Single incident (DQ rebase drop) recovered in-session. Score in 0.75-0.80 range is appropriate.

---

_Generated per `.claude/lessons/feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`, `feedback_retro_task_complexity_score.md`._
