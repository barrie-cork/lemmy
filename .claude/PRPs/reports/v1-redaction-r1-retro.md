# v1-redaction-r1 retro — GDPR identifier-scrubber hardening

**Sub-phase:** v1-redaction-r1 (harden `scrub`/`scrub_json` — adversarial tests, recursion cap, policy commentary)
**Branch:** `phase-v1-redaction-r1`
**Base:** `governance-v0` @ pre-cut tip
**Plan:** `.claude/PRPs/plans/v1-redaction-r1.plan.md` @ `5e9b1e35b` (complexity 4/10)
**Dates:** 2026-05-28 (planning + Tasks 0–3) → 2026-06-01 (e2e gate, fix-impl-1, CR, merge)
**Impl model:** four-role Junior orchestration (planning → impl-task × 3 → validate-pending-laptop × 3 → phase-tip e2e → bm-pr → CR → fix-impl-1 → bm-merge)
**PR:** #173 (`phase-v1-redaction-r1` → `governance-v0`) merged 2026-06-01 @ `8b8c3f6fe`
**Commits on phase branch (since governance-v0 fork):** ~35 (Tasks 0–3 + merge-forwards + DQ + fix-impl-1 + bm lifecycle + this retro)

---

## TL;DR for the advisor

**Plan delivered cleanly; one post-impl surprise (merge-forward clippy debt) required a fix-impl-1 cycle; one e2e run was lost to environmental failure before a clean pass.** All §16a stories shipped. The single-file sequential cohort (Tasks 1+2+3 all editing `redaction.rs`) was correctly structured — no index.lock contention, no `[P]` attempt. Two merge-forward incidents accumulated two LESSON trailers; both promoted below. CR was light (7 findings, all terminal — 1 rebutted, 6 wont-fix); no structural fix-in-PR cycle needed. bm-merge #569 failed due to governance-v0 advancing between brief-authorship and dispatch; resolved by merge-forward + re-dispatch (#570, succeeded in ~2 min).

One process gap identified: **merge-forward clippy debt is not caught by `bm-pr` pre-flight** — the first `bm-verify` failed on `reputation_snapshot.rs` clippy lints introduced by a quality-r3c merge into the workspace; the phase branch's own code was clean. Fix-impl-1 resolved in one Junior cycle.

---

## 1. What worked — keep doing

### 1.1 Single-file sequential cohort structure

Tasks 1, 2, and 3 all modified `redaction.rs`. The plan correctly marked none as `[P]` and the advisor serialised dispatch without hesitation. No index.lock contention, no diff collision, no post-merge rebase complexity. Wall-clock cost: Tasks 1+2+3 ran 2026-05-28 19:00–00:30 UTC (~5.5 hours for three sequential Tasks including validate-pending-laptop round-trips). This is the correct cost — parallel dispatch would have required three-way merge resolution for a single 159-line file.

**Keep**: the YAML-overlap check in §4.1 step 4 is load-bearing. When all `modifies:` entries converge on one file, the plan must say sequential and the advisor must not second-guess it.

### 1.2 validate-pending-laptop cadence

Each Task wrote a `validate-pending-laptop` DQ and stopped. The laptop advisor ran cargo check + clippy + targeted unit tests per task, got PASS × 3. The pattern worked without deviation. No forbidden-window violations; Tasks ran in the ~20:00–00:30 UTC window (clear of the Sunday NAS-backup window).

**Keep**: the write-DQ-and-stop discipline from `feedback_validate_pending_laptop_write_then_stop.md`. Workers did not invoke cargo themselves; no file-lock contention.

### 1.3 Conformance-audit prevention checkpoint fired correctly

The pre-planning conformance-audit (per §3.1.1) ran against `redaction.rs` before `/brehon-clarify`. Tier-1 result: zero findings (pure-Rust regex + tree-walk outside the governance framework's defect class). This was the expected result and the checkpoint took ~5 min — cost proportionate to the benefit (confirmed no ADR-violation risk before dispatching planning).

**Keep**: the §3.1.1 gate even for "obviously clean" files. The quick zero-finding result is itself load-bearing signal.

### 1.4 `#[ignore]` + `kind: log` carry-forward pattern

Two Unicode confusable test cases (ZWJ injection, in-username Cyrillic lookalike) shipped as `#[ignore = "v1-redaction-r2 will normalise via unicode-normalization crate"]` test fns. Task 1 also wrote a `kind: "log"` DQ recording the carry-forward. This is the correct shape for "we know the problem, we defer the fix, we don't lose the signal." The tests are visible in `cargo test` output as ignored, the DQ is auditable, and the TODO comment names the target sub-phase.

**Keep**: `#[ignore = "reason"]` + DQ log as the canonical deferral pair. Compared to comments-only or MEMORY.md entries, this shape is mechanically verifiable at retro time (grep `#\[ignore\]` in redaction.rs).

### 1.5 CR self-rebuttal pattern for scheduler-level guards

CR finding cr-6 challenged the `delete_older_than` fn in e2e.rs for lacking an env-var guard. The advisor's rebuttal comment clarified that the guard is at the scheduler level (the `DELETE_OLD_REPORTS` env var gates the *scheduler call*, not the function itself); the function is a pure DB operation. CR accepted on second pass and self-rebuted. No code change needed; a one-line comment clarification (`75ecec03d`) was sufficient.

**Keep**: the clarify-via-comment path before treating a CR finding as fix-in-PR. The cost of a comment is ~5 min; avoiding a wrong fix-in-PR is ~45 min.

---

## 2. Per-role signals

### 2.1 Advisor

**What worked:** Brief series (impl-0 through impl-3) was crisp — each brief was authored, committed to governance-v0, then SSH-merged into the phase branch (Mode A + Mode B hybrid). The single-file scope meant each brief was short. DQ entry flow was clean: impl wrote validate-pending-laptop entries, advisor ran laptop cargo and mutated to PASS, advance; no entries required user escalation during the impl phase.

**Merge-forward discipline gap:** The advisor ran two merge-forwards (pre-bm-pr and pre-bm-merge). Both revealed the same class of issue: governance-v0 advancing introduces changes (quality-r3c clippy fixes, RT-r5 e2e test updates) that interact with the phase branch. The pre-bm-pr merge-forward (2026-06-01 01:28) correctly pulled RT-r5 e2e changes but the e2e.rs conflict resolution **took the wrong side** (kept phase-branch version of DISABLE_* test placement instead of governance-v0 version). This was caught by the e2e fail at `1b88b91ce` and manually patched at `9bf49615f`. The advisor should default to governance-v0 side for e2e.rs conflicts when the phase branch made no functional changes to that file — a LESSON trailer captured this (see §5).

**bm-merge #569 fail → re-dispatch:** governance-v0 advanced 10 commits between brief authorship and #569 dispatch. This is a known failure mode but the timing was unexpected — quality-r3c merged while #569 was queued. The fix (merge-forward + re-dispatch #570) took ~15 min. Pre-dispatch merge-forward check (not just at brief time) would have caught it immediately.

**User gates:** Gates 1 (plan approval) + 3 (CR triage) + 4 (e2e local vs dispatch) + 5 (merge confirm) all fired cleanly. Gate 4 was implicit (laptop-native, no dispatch choice needed). Gate 3 required one user interaction (triage counts surfaced, user confirmed approve).

### 2.2 Planning

**Brief sufficiency:** Planning task #485 produced a complete, actionable plan with correct MIRROR refs, per-task YAML file arrays, and a clean §5.1 complexity score (4/10). No clarify-DQ escalated to user. Plan §15 DoD smoke-test commands executed as written (R8 exception for targeted `-p lemmy_db_schema` form documented and observed by impl). No phantom dependencies.

**WP-6 call-site audit:** Planner enumerated 23 `scrub`/`scrub_json` call sites at plan-author time; Task 0 Probe 6 re-enumerated and confirmed count within ±2 tolerance. The audit correctly identified zero bypass paths into the three gated columns.

**One planning accuracy gap:** The brief §9 MIRROR refs listed `redaction.rs:1-25` for the module-doc shape. Post-Task 2, the file grew by ~30 lines (bounded recursion rewrite) and the line ranges in the Task 3 brief needed small adjustments. The impl session handled this without a DQ blocker — impl re-read the file at task start per §9 "every task Reads redaction.rs:1-159 end-to-end BEFORE first edit." No incident, but the line-range citations in Task 3's brief could have been fresher.

**Keep**: the plan's pre-task "MIRROR: re-read file before editing" instruction. It absorbed the line-number drift without escalation.

### 2.3 Impl-task

**Tasks 1, 2, 3 all completed in single runs** — no failed first run, no mid-task blocker DQ. Timings:
- Task 1 (adversarial tests): 2026-05-28 21:01–21:12 UTC, ~11 min worker time, 1 commit, 1 file
- Task 2 (bounded recursion): 2026-05-28 23:40–23:51 UTC, ~11 min worker time, 1 commit, 1 file
- Task 3 (commentary): 2026-05-29 00:17–00:26 UTC, ~9 min worker time, 1 commit, 1 file
- fix-impl-1 (clippy): 2026-06-01 09:14–09:19 UTC (inferred), ~5 min worker time, 1 commit, 1 file

**MIRROR-ref accuracy held:** The existing test pattern (plain `#[test] fn`, `assert_eq!`, `pretty_assertions` already imported) was mirrored verbatim per §10.1. No `LemmyResult` substitution, no `async`, no import drift. The depth-cap test (§10.3) walked 64 levels correctly.

**One impl-task lesson surface:** fix-impl-1 touched `reputation_snapshot.rs` rather than `redaction.rs` — a pre-existing clippy debt surfaced by the quality-r3c merge into the workspace. The impl-task correctly identified the debt as pre-existing (not introduced by r1) and fixed it cleanly with `map_or→is_some_and` + `as`→`TryFrom` conversions. The brief correctly scoped fix-impl-1 to reputation_snapshot.rs only; impl stayed in scope.

### 2.4 BM

**CR was light:** PR #173 received 7 CR findings. All 7 were terminal in the first triage pass (cr-1 through cr-5 wont-fix; cr-7 wont-fix; cr-6 rebut path via comment clarification). No fix-in-PR cycle required (fix-impl-2 was authored for cr-6 as a precaution but superseded when CR self-rebutted). BM triage was clean; four-bucket distribution was accurate; findings YAML `final_recommendation: approve` was correct.

**bm-merge #569 DIRTY failure:** The first bm-merge task failed with `mergeStateStatus DIRTY/CONFLICTING` because governance-v0 had advanced 10 commits. This is a known pattern (`feedback_bm_merge_unstable_admin_bypass.md`) but DIRTY (not just UNSTABLE) requires a merge-forward, not just --admin bypass. The BM task correctly stopped rather than force-merging. The advisor recognised the merge-forward path and resolved within one session.

**Runlog completeness:** bm-runlog.md captured all BM state changes. The stash pop after merge-forward left bm-runlog.md modified locally (uncommitted) — this is a cosmetic miss; the file contents were current, just not committed at handover time. No information lost.

---

## 3. What to change

### 3.1 Pre-bm-pr verify should include merge-forward clippy check

The first `bm-verify` at `47bff187a` failed because quality-r3c introduced `reputation_snapshot.rs` clippy lints into the workspace that the phase branch had not triggered before. The verify command ran `cargo clippy --workspace` which caught this, but only after `bm-pr` was already queued. A pre-bm-pr step that runs `cargo clippy --workspace --features full --no-deps -- -D warnings` against the post-merge-forward HEAD would have caught this before the bm-pr brief was authored — saving one Junior cycle.

**Proposed change:** add a `validate-pending-laptop` DQ for `cargo clippy --workspace` immediately after every merge-forward that introduces quality-r3c or similar lint-fix commits. The advisor's merge-forward commit body should note "lint-fix commits in incoming" as a trigger for an extra pre-bm-pr clippy validate.

### 3.2 Merge-forward e2e.rs conflict: default to governance-v0

When a merge-forward produces a conflict on `crates/server/tests/e2e.rs` AND the phase branch made no functional changes to that file (only advisory changes like Task 1's DISABLE_* test additions), the correct resolution is **take governance-v0 side**. The advisor took the phase-branch side at `fd01f6ef9` (pre-bm-pr merge-forward), which kept the wrong DISABLE test placement introduced by the quality-r3c merge. This was caught by the e2e run failure at `1b88b91ce` and fixed at `9bf49615f`. Cost: ~1 hour (e2e run + diagnosis + patch + re-run).

**Promoted as LESSON below.** Short rule: in a merge-forward, for e2e.rs conflicts where the phase branch only appended comments/DISABLE attributes (no test logic changes), take governance-v0 unless you can verify the phase side is strictly a superset.

### 3.3 bm-merge pre-dispatch: always re-check governance-v0 freshness

The advisor checked merge-forward status when authoring the bm-merge-1 brief, but the 15-min delay between brief commit and task dispatch was enough for quality-r3c to land on governance-v0. The pre-dispatch check should happen immediately before `create_task`, not at brief-authorship time. The bm-merge brief template already includes "verify governance-v0 is up to date" in its §4 constraints — the gap was that the advisor ran this check ~15 min before dispatch rather than immediately before.

**Proposed change:** after authoring any bm-merge brief, the advisor should run `git ls-remote origin governance-v0` immediately before `create_task` and compare to the brief's stated `base_sha`. If they differ, update the brief before dispatch.

---

## 4. Per-task complexity scores

| Task | Files | Commits | Runtime (min) | Max log silence (min) | Notes |
|---|---|---|---|---|---|
| T0 (pre-flight audit) | 0 | 0 | ~8 | <8 | Junior #484; no commit, probes only |
| T1 (adversarial tests) | 1 | 1 | ~11 | <11 | `e42cb4d70` |
| T2 (bounded recursion) | 1 | 1 | ~11 | <11 | `471ba7565` |
| T3 (commentary) | 1 | 1 | ~9 | <9 | `71709af59` |
| fix-impl-1 (clippy) | 1 | 1 | ~5 | <5 | `df5bf3558` |

All tasks well under the 55-min runtime / 40-min log-silence / 8-file ceilings. The phase was a true complexity-4 execution.

**Phase totals:** 5 impl files touched (all in `redaction.rs` + one in `reputation_snapshot.rs`), 5 impl commits, ~44 min cumulative worker time.

---

## 5. Lessons promoted this phase

Two LESSON trailers in phase branch commits; both promoted to `.claude/lessons/`:

### feedback_merge_forward_clippy_debt_from_trunk.md

**Source:** `8b272783b` (advisor), fix-impl-1 brief commit body
**LESSON trailer:** "merge-forward from governance-v0 can surface clippy debt from prior phases that compiled fine before but now trips -D warnings in the updated workspace; check brehon-verify after every merge-forward."

**Promoted rule:** after every merge-forward that pulls quality-r* or lint-fix commits from governance-v0, run `brehon-verify` (or at minimum `cargo clippy --workspace --features full --no-deps -- -D warnings`) before queueing the next Junior task. The phase branch may be clean; the workspace may not be.

### feedback_merge_forward_e2e_conflict_default_to_governance.md

**Source:** `fd01f6ef9` (merge commit body)
**LESSON trailer:** "merge-forward conflicts on e2e.rs should default to governance-v0 side when the phase branch made no changes to that file."

**Promoted rule:** when a merge-forward produces a conflict on `crates/server/tests/e2e.rs` and the phase branch's only changes to that file are appended comments, `#[ignore]` attributes, or DISABLE_* markers (no test-logic edits), take the governance-v0 side of the conflict. The governance-v0 side reflects the latest correctly-integrated test configuration. Taking the phase side risks carrying stale DISABLE placements or conflict artifacts into the branch.

---

## 6. What to carry forward

1. **v1-redaction-r2 Unicode confusable tests** — `#[ignore]`'d tests `scrub_zero_width_joiner_between_at_and_handle` and `scrub_handle_with_unicode_confusable_in_username` in `redaction.rs`. Deferred pending `unicode-normalization` crate dependency. Tracked via DQ log entry from Task 1 and `#[ignore = "v1-redaction-r2 will normalise via unicode-normalization crate"]` reason strings.

2. **Pre-bm-pr clippy gate after merge-forward** (§3.1) — add a `validate-pending-laptop` for `cargo clippy --workspace` immediately after any merge-forward that brings lint-fix commits. Not a rule-file change; an advisor discipline to apply from the next phase onward.

3. **bm-merge brief: immediate pre-dispatch governance-v0 freshness check** (§3.3) — run `git ls-remote origin governance-v0` after brief authorship and immediately before `create_task`. If SHA differs from brief's `base_sha`, update the brief first.

4. **Task-3 brief line ranges drift** (§2.2 planning gap) — brief templates that cite specific line ranges in files being progressively edited across Tasks 1+2+3 will drift by Task 3. Mitigation: the "re-read file before editing" instruction in each brief absorbs the drift; no new rule needed. But note in the next multi-task single-file plan that the Task N brief's MIRROR refs should cite relative positions ("after the `scrub_json` fn" rather than "line 88–101") for robustness.
