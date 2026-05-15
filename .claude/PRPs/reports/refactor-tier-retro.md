# Refactor-tier retro — tonight's parallel lane (PR-4 / PR-5 / PR-6)

**Scope:** the 3 parallel-lane refactor PRs of the fix-before-next-phase tier
(`refactor-execution-plan-2026-05-14.md` Step 2c cohort). Driven autonomously
overnight, strictly sequential per user instruction ("I prefer to do
seqnetially"). NOT a completion report — what surprised us, what to change.

**Author:** advisor (governance-v0 canonical session)
**Date:** 2026-05-15
**Triggering instruction:** "After completion run retro and stop. wait for
further instructions."

---

## TL;DR for the advisor

- **3/3 tonight's lanes merged:** PR-4 #128 (valid-from), PR-5 #129
  (diesel-errors), PR-6 #130 (seed-tests). All squash-merged into
  `governance-v0`, branches deleted, worktrees removed.
- **Strict gate is NOT 5/5.** Tonight's scope was the Step-2c parallel
  cohort only (PR-4/5/6). **PR-1** (e2e error-types, L-effort, dedicated
  serial lane, ranks 1/2/7/8) and **PR-2** (TOCTOU `create_report.rs`,
  M-effort, dedicated serial lane, rank 3) remain **unstarted** — they are
  the Step-2a/2b serial lanes, separate scope, NOT tonight's work. The
  strict gate to v1 PRD planning (`refactor-execution-plan-2026-05-14.md`
  §"Strict gate" + line 86 "All 5 PRs merged") is **3/5 — NOT satisfied**.
- **Zero PR-6 code defects.** All 3 obstacles this lane were non-code
  (stale daemon base, missing submodule, wrong brief package-selector).
  PR-6's actual deliverable (5 `parse_founder_spec` tests) passed clean.
- **The headline surprise:** "sequential & autonomous & context-saving"
  worked, but the cost was *not* the impl work — it was the **recovery
  overhead from upstream defects** (planning-brief inaccuracies, worktree
  bootstrap gaps, persistent CI infra failure). 6 distinct lessons, none
  of them about the refactor code itself.
- **Positive four-role signal:** impl #266 *improved on* the planning
  brief (caught a clippy-unsafe example and fixed it autonomously). The
  role separation paid off — impl is not a stenographer.

---

## Per-task complexity table

Per `feedback_retro_task_complexity_score`
(`files / commits / runtime-min / max-log-silence-min`). Runtime =
dispatch→merge wall-clock for the lane (incl. recovery); commits = on the
merged squash's source branch pre-squash.

| Lane | Junior | Audit | files | commits | runtime-min | max-log-silence-min | Notes |
|---|---|---|---|---|---|---|---|
| PR-4 valid-from | #263 | 3.D.6 (rank 6) | 2 migration | ~2 (code + DQ-backfill) | (prior-session; ~hrs incl. 2× stuck-runner) | n/a (advisor-laptop) | Recipe-1 validate-pending DQ **omitted** by worker; advisor backfilled. 2× GH stuck-runner. |
| PR-5 diesel-errors | #265 | 3.A.5 (rank 13) | 1 (`admin_audit_stream.rs`) | ~2 (code + DQ-backfill) | (prior-session; ~hrs incl. 1× stuck-runner + force-push CR re-settle) | n/a | Recipe-1 DQ **omitted** by worker; advisor backfilled. CR force-push re-settle (#129 CONFLICTING precursor → L5). |
| PR-6 seed-tests | #266 | 3.E.20 (rank 17) | 1 (`seed_founders/src/main.rs` +75) | 3 (code + DQ #218 + runlog) | ~120 (this session, incl. submodule fix + selector fix + full cargo-check 6m21s + test compile) | ~7 (test-profile dep recompile silence; healthy) | Worker raised **colliding** DQ #214 off stale daemon base — discarded; re-authored fresh #218. Submodule L6. Selector L6b. |

**Reading:** the merged code in all 3 was tiny (S-effort, ≤2 files,
pure-additive or isolated). 100% of the wall-clock overhead was recovery
from the 6 lessons below — none from the refactor logic.

---

## Per-role signals

Per `feedback_four_role_retro_signals` — one H2 per role.

### Advisor (this session — orchestrator)

**Worked:**
- §5.2 advisor-laptop fallback held under the 4th-consecutive GH
  stuck-runner. User-authorised "Go with C" generalised cleanly across
  PR-4/5/6; the local `cargo-check.bat --workspace --features full` →
  `CHECK_EXIT_0` gate is a reliable substitute when Shape-G is broken.
- Single-accurate-forward-wakeup discipline (replace, not accumulate)
  fixed the stale-wakeup-noise problem the user flagged. After the
  CronDelete of the duplicate, the cron state stayed clean for the rest
  of the run.
- Honest checkpoint-retro on every mid-loop Stop-hook fire (evals
  318/319/320/321, `source_ref=governance-v0`, ~0.6, explicitly
  "checkpoint not task-complete"). Zero forge / hook-mod / raw-SQL.
- DQ #218 written **honestly** — recorded BOTH the 5/5 pass AND the 8
  unrelated `lemmy_api` env-failures, rather than the brief's anticipated
  clean "TEST_EXIT_0 5-passed". Surfaced the validation-narrative split
  to the user as a gate rather than auto-deciding it.
- L5 applied correctly: the bm-pr brief stayed on `governance-v0`, never
  cherry-picked onto the chore branch → zero CONFLICTING (the #129 root
  cause did NOT recur).

**Surprised / to change:**
- The stale-wakeup-noise class recurred multiple times before root-caused
  (duplicate ScheduleWakeup-created crons racing). The fix is now a
  standing discipline; it should be a **rule**, not a per-session habit
  (proposed L7 below).
- Polling cadence: ~10 polls across the PR-6 lane, each re-tailing a
  growing cargo log. Acceptable but the cargo-test wrapper's `--workspace`
  override (vs `-p` scoping) made the log noisy with whole-workspace
  recompile and surfaced 8 false-alarm failures that needed careful
  signal/noise separation. The wrapper behaviour is a latent trap (L6b
  sub-finding).

### Planning (the impl/bm briefs — authored pre-tonight)

**Worked:**
- Brief structure (§Scope / §Required reading / §Constraints / §Out of
  scope) was followed by every Junior; the file-ownership boundaries held
  (no worker touched outside its lane).
- The bm-pr brief's explicit L5 note ("this brief lives ONLY on
  governance-v0 — NOT cherry-picked onto chore branch") was load-bearing
  and prevented a #129-class CONFLICTING recurrence on PR-6.

**Surprised / to change (L6b — 2 planning-brief defects this lane):**
1. **Wrong cargo `-p` selector.** `refactor-seed-tests-impl.md` §2.5/§4
   prescribed `-p seed_founders`, but the crate name is
   `brehon_seed_founders` (directory name ≠ crate name). cargo-test
   attempt-1 failed "package not found". The brief assumed dir==crate
   without reading `crates/tools/seed_founders/Cargo.toml`.
2. **Clippy-unsafe test example.** §2.3's example used `.expect()` /
   `.unwrap_err()` — which the workspace clippy config denies even in
   test code (per `feedback_clippy_test_style`). Had the worker copied it
   verbatim it would have failed clippy.
   → **Proposed fix:** planning briefs that name a cargo `-p` target MUST
   `grep '^name' <crate>/Cargo.toml` and cite the real package name;
   test-example snippets MUST be clippy-safe (`LemmyResult<()>` + `?`,
   never `.expect()`/`.unwrap_err()`).

### Impl (Junior workers #263 / #265 / #266)

**Worked — POSITIVE four-role signal:**
- Impl #266 **improved on the planning brief**: it recognised §2.3's
  example was clippy-unsafe and *deviated*, writing `LemmyResult<()>` +
  `?` with a private `assert_unknown_err` helper instead. This is exactly
  the role-separation value-add the four-role model is for — impl is a
  reasoning agent, not a stenographer. The 5 tests are idiomatic and CR
  found **zero** to flag on them.

**Surprised / to change (L1 — two distinct DQ failure modes, NOT a clean
3/3-omit):**
- **2/3 omitted** the mandatory Recipe-1 `validate-pending` DQ entirely
  (#263 PR-4, #265 PR-5) — advisor backfilled both.
- **1/3 raised it but off a stale base** (#266 PR-6): the daemon's
  `/srv/brehon-fork` `governance-v0` was stale at task-create, so #266
  branched off `f0c2b75af` (carrying already-merged PR-4/PR-5 commits)
  and computed `next_id` against its stale view → raised a **colliding
  DQ #214** (its max was 214; the real max was 217). Recovered by
  cherry-picking ONLY the real code commit and re-authoring the DQ fresh
  as #218.
  → These are **two different bugs** (omit vs stale-base-collision), not
  one. Proposed fixes: (a) impl-task contract hard-gate requiring the
  validate-pending DQ + `next_id` recompute against **ORIGIN** refs (not
  the worker's local view); (b) daemon `/srv/brehon-fork` pre-dispatch
  `git fetch && git reset --hard origin/governance-v0` — this is the
  **root** of the stale-base class.

### BM (bm-pr — run INLINE by advisor per L3/L15)

**Worked:**
- Advisor-inline bm-pr is now the **standard** for chore-branch lanes.
  Junior `bm-task` with `base_branch=chore/*` fails on daemon worktree-ref
  resolution (#264 PR-4, confirmed PR-5). Running the read-only gate work
  + `gh pr create` inline (advisor already has the context loaded)
  eliminated the duplicate-context-boot and the daemon ref-fail entirely.
  PR #130 opened first-try.
- Findings YAML discipline held: `pr-130-findings.yaml` created per
  SCHEMA.md, correctly NOT committed (gitignored runtime artifact).

**Surprised / to change (L3):**
- The bm-pr brief **template** still implies a Junior `bm-task` dispatch.
  It should be updated to state advisor-inline as the default for
  chore-branch lanes (the Junior path is dead for `base_branch=chore/*`).

### ci-watcher (Shape-G validation)

**Did not run this lane — and that is the finding (L2).**
- The GH `cargo-validate-workspace` workflow was a **persistent stuck
  runner**: 4 instances across the tier (PR-4 ×2, PR-5 ×1, PR-6 ×1),
  every one frozen `updatedAt`-since-creation with zero job progress.
  ci-watcher never got a real run to poll; the advisor early-tripped each
  (~8-10 min) and fell back to §5.2 advisor-laptop.
- → **Proposed fix:** add `timeout-minutes` + `concurrency:
  cancel-in-progress` to `cargo-validate-workspace.yml`, and codify the
  advisor early-trip (~8-10 min frozen-updatedAt → cancel + §5.2) as a
  rule. 4/4 stuck is not flaky — it is broken for this repo's runner
  config and needs an infra fix, not just a fallback.

---

## Consolidated lessons (with occurrence counts)

| ID | Lesson | Occurrences this tier | Proposed durable fix |
|---|---|---|---|
| **L1** | impl-task DQ discipline has TWO failure modes: (a) omit Recipe-1 validate-pending DQ; (b) raise it off a stale daemon base → colliding `next_id` | 3/3 (#263 omit, #265 omit, #266 stale-base-collision) | impl-task contract hard-gate: validate-pending DQ mandatory + `next_id` recompute vs **ORIGIN** refs. **Root fix:** daemon `/srv/brehon-fork` pre-dispatch `git fetch && git reset --hard origin/governance-v0`. |
| **L2** | GH `cargo-validate-workspace` persistent stuck-runner (frozen updatedAt, 0 job progress) | 4 (PR-4 ×2, PR-5 ×1, PR-6 ×1) | Workflow `timeout-minutes` + `concurrency: cancel-in-progress`; codify advisor early-trip (~8-10 min → cancel + §5.2) as a rule. |
| **L3** | Junior `bm-task` with `base_branch=chore/*` fails on daemon worktree-ref resolution | 2 confirmed (#264 PR-4, PR-5) | Advisor-inline bm-pr is STANDARD for chore lanes. Update the bm-pr brief template. |
| **L4** | Stop hook fires mid-loop on `governance-v0` (time-window mode) | every poll-tick this session | Honest checkpoint-retro (`source_ref=branch`, ~0.6, "checkpoint not task-complete") is the correct path. Evals 318/319/320/321. NEVER forge/modify-hook/raw-SQL. (Already established; reaffirmed.) |
| **L5** | Cherry-picking the bm-pr brief onto the chore branch causes CONFLICTING (the #129 root cause) | 0 this lane (APPLIED OK on PR-6) | Keep the bm-pr brief on `governance-v0` only; reference by path. Already in the PR-6 brief frontmatter — worked, zero conflict. |
| **L6** | Re-pointed git worktree does NOT auto-init submodules → `crates/email/translations` (lemmy-translations) uninit → `lemmy_email` `build.rs` `read_dir` fails | 1 (PR-6 cargo-check attempt-1) | Per `feedback_worktree_submodules_not_auto_init`. Add `git submodule update --init [--recursive]` to `multi-lane-worktree.md` §"Lifecycle" Step 1 worktree-add ritual. |
| **L6b** | Planning-brief inaccuracies: (a) wrong cargo `-p` package name (`seed_founders` vs `brehon_seed_founders`, dir≠crate); (b) clippy-unsafe `.expect()`/`.unwrap_err()` test example §2.3 | 2 (this lane) | Planning briefs naming a cargo `-p` target MUST `grep '^name' <crate>/Cargo.toml`; test-example snippets MUST be clippy-safe (`LemmyResult<()>`+`?`). |
| **L7** (new) | ScheduleWakeup-created crons accumulate → stale wakeup re-injects superseded procedure | recurred multiple times this run | Codify "exactly one accurate pending wakeup per cycle (replace via CronDelete, never accumulate)" as advisor-orchestrator discipline. |

**Positive (preserve, do not 'fix'):** impl #266 deviated from a defective
planning-brief example to the correct clippy-safe pattern autonomously.
The four-role separation worked as designed — flag *planning-brief example
accuracy* (L6b) as the forward improvement, NOT impl behaviour.

---

## What did NOT need fixing (worth preserving)

- The refactor code itself: PR-4 (2 migration literals pinned), PR-5
  (1-site Diesel error propagation via `match`+`continue`), PR-6 (5
  `parse_founder_spec` tests) — all correct, CR-clean on the code.
- Sequential discipline: PR-4 → PR-5 → PR-6, strictly one at a time per
  user preference. No parallelism-induced collision (the per-lane
  worktree topology + serial gate held).
- The 6 user gates: every one surfaced and honoured (validation-narrative
  gate on PR-6 added beyond the standard 6 because the honest validation
  story was more nuanced than the brief anticipated). No auto-decide.
- Honest DQ attribution: `advisor-laptop` for §5.2; commit subjects
  matched `^chore\((decision-queue|bm|advisor)\)`.

---

## Quantified outcomes vs confidence

| Signal | Value |
|---|---|
| Tonight's lanes merged | **3/3** (PR-4 #128, PR-5 #129, PR-6 #130) |
| Strict gate (5 PRs) | **3/5 — NOT satisfied** (PR-1 + PR-2 are separate serial lanes, unstarted) |
| PR-6 code defects | **0** |
| Non-code obstacles recovered | **3** (stale base, submodule, selector) |
| Distinct lessons | **7** (L1-L6b + new L7); 0 about refactor logic |
| CR findings on the code | **0 actionable** (PR-6 cleanest of 3; PR-4 had 2, PR-5 had 5) |
| Process breaches | **0** (no forge/hook-mod/raw-SQL; no auto-merge; no gate skip) |
| Positive role signals | **1** (impl #266 improved on a defective brief) |

**Pre-retro confidence (gut, before scoring):** 0.78 — the deliverables
all merged cleanly and process held; the drag was entirely upstream-defect
recovery, all root-caused with proposed fixes.

---

## Suggested action items for the advisor (NOT executed — awaiting sign-off)

These are **proposals only**. Per the user's instruction
("After completion run retro and stop. wait for further instructions") I
have NOT applied any of them and will NOT proceed to v1 PRD planning,
`/brehon-phase-transition`, or any further dispatch.

1. **L1 root fix (highest value):** add daemon `/srv/brehon-fork`
   pre-dispatch `git fetch && git reset --hard origin/governance-v0` to
   the Junior task-create path. This kills the stale-base class outright
   (it caused #266's colliding DQ #214 and is latent on every future
   lane). Plus an impl-task-contract hard-gate for the validate-pending
   DQ + ORIGIN-ref `next_id`.
2. **L2 infra fix:** `timeout-minutes` + `concurrency: cancel-in-progress`
   on `cargo-validate-workspace.yml`; codify advisor early-trip rule. 4/4
   stuck means the workflow is broken for this runner config, not flaky.
3. **L6 / L6b / L3 / L7 doc fixes:** worktree-add submodule-init ritual
   in `multi-lane-worktree.md`; planning-brief cargo-`-p`-grep +
   clippy-safe-example rule; bm-pr-brief-template advisor-inline default;
   one-accurate-wakeup discipline in `advisor-orchestrator.md`.
4. **Strict gate remains OPEN:** PR-1 (e2e error-types, L) and PR-2
   (TOCTOU, M) are unstarted serial dedicated-session lanes. v1 PRD
   planning does NOT resume until those 2 also merge (gate = 5/5, per
   `refactor-execution-plan-2026-05-14.md` line 86). User decides when /
   whether to start them.

---

**STOP.** Autonomous loop halted. Retro surfaced. Awaiting user
instructions. No ScheduleWakeup. No post-retro action.
