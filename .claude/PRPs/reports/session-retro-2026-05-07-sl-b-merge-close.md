# Session retro — 2026-05-07 — sl-b-merge-close

**Harness:** claude-code (advisor session)
**Session window:** ~2026-05-07T14:30Z → 20:43Z (~370 min)
**Branch at start:** `b4718e4de` (`phase-v1-SL-b`, DQ #155 ci-watcher pending)
**Branch at end:** `3462bd35f` (`governance-v0`, PR #119 merged)
**Files touched:** 51 (via merge commit `9ae4c332c`)
**Commits:** ~30 advisor/BM commits on phase-v1-SL-b + 1 merge commit on governance-v0

## TL;DR

Session drove v1-SL-b from "ci-watcher #134 in flight" to "PR #119 merged on governance-v0." The critical path was: poll ci-watcher #134 (pass) → Phase 2 e2e tip 620861f08 (FAIL — sponsor_pseudonym assertion) → §G4 triage → fix-impl-4 (3 string literal renames in e2e.rs) → ci-watcher #136 (pass) → Phase 2 e2e tip c87179af4 (76/76 pass) → bm-poll-cr #137 → bm-triage (Opus subagent, recommend approve) → conflict-resolve merge-into-governance-v0 → PR #119 merged. Top change proposal: add a lesson for the "field rename in handler → grep e2e.rs for old name" footgun that caused the fix-impl-4 detour.

---

## What surprised us

**Advisor:**
- The e2e test failure on tip `620861f08` was a genuine surprise — cr-7b task #133 renamed `sponsor_pseudonym` → `revoker_pseudonym` in the handler but the brief scoped changes to `revoke_endorsement.rs` only. The e2e assertions were not in scope and not caught by the workspace-check Phase 1 (clippy/check passes even with stale string literals in test assertions). Shape G Phase 1 + Phase 2 validation split exposed this correctly — a purely workspace-check gate would have shipped broken e2e.
- The bm-merge subagent (task #138) ran without finding the brief because the EliteDesk hadn't fetched the latest `phase-v1-SL-b` commits yet. The brief was committed and pushed before queuing, but the daemon branches from its local HEAD — it doesn't auto-fetch. Required: manual `git pull` on EliteDesk then re-queue. This is the **fourth occurrence** of the daemon-not-fetching pattern (`feedback_junior_daemon_finalize_skips_when_worker_pre_pushes` family).
- `git rebase origin/governance-v0` across 86 phase-branch commits with many DQ-touching commits produces one conflict per DQ commit (86 rounds of conflict resolution). The merge approach (one `git merge` + two `--ours` resolutions) was orders of magnitude faster. Rebase is the wrong tool when the conflict file touches every commit in the history.
- The adr-compliance bypass mechanism (owner "acknowledge" comment in PR) silently failed to satisfy the gate on the latest merge-commit push, even though prior ack comments existed. Root cause unclear (possibly a timing issue with the GitHub API comment fetch in the workflow script). Posted a fresh ack comment + empty re-trigger commit; the scan still failed. User chose to merge directly past the advisory check.

**BM:**
- bm-triage subagent (Opus, background, task `aee6ad9a`) produced correct bucket assignments but returned via result summary rather than AskUserQuestion (subagent doesn't have that tool). Parent session had to relay the confirm prompts. This is expected behavior — documented pattern, not a bug.

**Impl:**
- fix-impl-4 (task #135) ran clean: 4 min total, 1 commit, 3 string literal changes. Smallest impl-task this sub-phase. Complexity score: `1/1/4/2`.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add `.claude/lessons/feedback_handler_field_rename_grep_e2e.md`: "when a handler payload field is renamed, grep `crates/server/tests/e2e.rs` for the old field name before committing" | Prevents §G4 fix-impl detour on field renames; e2e catch moved from Phase 2 (26-min run) to brief-write time | minor | 1× this session (sl-b cr-7b); probable recurrence on any field rename |
| 2 | Add EliteDesk pre-queue fetch step to `/precheck` skill: `ssh homeserver 'cd /srv/brehon-fork && git fetch origin <branch> && git merge --ff-only origin/<branch>'` before any bm-task brief-queuing | Prevents task #138 "brief not found" failure caused by daemon HEAD lagging behind laptop push | minor | 2× this session (task #138 re-queue; systemic pattern in `feedback_junior_daemon_finalize_skips_when_worker_pre_pushes`) |
| 3 | Update bm-merge brief template to include an explicit note: "if mergeStateStatus=DIRTY, use `git merge origin/governance-v0 --no-commit --no-ff` + `checkout --ours <conflicted-files>` from the laptop; rebase is wrong when the phase branch has many DQ-touching commits (N conflicts = N commit rounds)" | Saves 10-15 min of wrong rebase attempt on future phase merges | minor | 1× this session; but applies to every phase-close |
| 4 | Add lesson or PR-template note: "adr-compliance advisory bypass (`acknowledge` keyword in owner comment) may not satisfy the gate on a merge commit push if the comment was posted on an earlier HEAD — post a fresh ack comment AND an empty re-trigger commit before re-running" | Prevents repeated CI debugging on a known advisory false-positive | minor | 1× this session; prior ack comments existed but didn't satisfy |

## What to carry forward

- **Shape G two-phase validation caught a real bug.** Phase 1 (workspace-check) passed while Phase 2 (e2e 76 tests) caught the stale `sponsor_pseudonym` assertion. The split is working as designed — don't compress these into a single gate.
- **bm-triage as background Opus subagent works well.** 35 min runtime, 15 findings correctly bucketed, recommendation flipped to `approve`, digest comment drafted. The pattern of: subagent returns result → parent session relays AskUserQuestion → confirms outbound actions is clean. The only gap is the AskUserQuestion tool availability in the subagent; relay pattern is acceptable.
- **`git merge --no-commit --no-ff` + `checkout --ours` for phase-close conflict resolution.** Resolves 2 known conflicts in one operation vs 86 rebase rounds. This is the correct approach for phase branches with diverged meta-commit histories (DQ entries, advisor chore commits) that don't conflict on code.
- **DQ #156 (historical fail record) stays in `pending[]` per option-2 rules** and is NOT blocking. Future sessions: note this in bm-merge brief so the BM subagent doesn't treat it as a blocker. The brief now has an explicit call-out; carry this pattern to future sub-phases.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| ci-watcher #134 (task) | 23 | 0 | none | Polled `25502392204`, mutated DQ #155 pass cleanly |
| Phase 2 e2e tip 620861f08 (local, ~26 min) | 0 | 0 | high | Correctly caught stale `sponsor_pseudonym` assertion — Shape G working as designed |
| §G4 triage → fix-impl-4 brief | 5 | 0 | none | Allowlist match (string literal rename); narrow fix correctly scoped |
| impl-task #135 fix-impl-4 | 15 | 0 | none | 4 min runtime; mechanical string renames; clean |
| ci-watcher #136 (task) | 3 | 0 | none | 1.5 min runtime; pass |
| Phase 2 e2e tip c87179af4 (local, ~26 min) | 0 | 0 | none | 76/76 pass; confirms fix |
| bm-poll-cr #137 (task) | 10 | 0 | none | Clean re-poll; 15 findings ingested |
| bm-triage Opus subagent (background) | 35 | 0 | low | Correct buckets; relay-confirm pattern worked; AskUserQuestion gap expected |
| bm-merge #138 (brief not found) | 0 | 8 | medium | EliteDesk hadn't fetched; re-queue after manual pull |
| bm-merge #139 (rebase attempt) | 0 | 15 | high | 86-commit rebase wrong approach; aborted; `git merge --ours` correct |
| Direct `gh pr merge` (advisor session) | 10 | 5 | low | adr-compliance advisory bypass failed silently; user chose direct merge |
| `/precheck` skill | 3 | 0 | none | All 5 checks clean; correctly surfaces EliteDesk branch |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) | Risk |
|---|---:|---:|---:|---:|---|
| fix-impl-4 (#135) | 1 | 1 | 4 | 2 | low |
| bm-poll-cr #137 | ~5 YAML | 3 | 3 | 1 | low |
| bm-merge #138 (failed) | 0 | 0 | 3 | 1 | n/a |
| bm-merge #139 (failed — DIRTY) | 0 | 0 | 3 | 1 | n/a |
| Phase 2 e2e (local x2) | 0 | 0 | ~52 (26×2) | ~26 | expected |

All tasks well within watchdog envelope. Phase 2 e2e dominates wall-clock at 52 min total but that's expected — two full e2e runs on a 76-test suite.

## Decisions to revisit

- **adr-compliance bypass reliability:** the `acknowledge` keyword check in `.github/scripts/adr-compliance.sh` didn't satisfy the gate on the merge-commit push despite prior ack comments existing. Worth reading the bypass logic with a test case — does it check for the *most recent* comment, or any comment? If it checks latest only, a fresh ack on every phase-tip push is required.
- **BM auto-merge in low-risk situations (critical=0, recommendation=approve, advisory-only CI failure):** user suggested BM could auto-merge without waiting for user confirm in this case. Draft a DQ entry or brief-addendum protocol for this after v1-SL-c ships.
- **EliteDesk pre-queue fetch:** pre-check step 3 already surfaces git state but doesn't auto-fetch. Add an explicit note or step to `/precheck` output: "if queueing a bm-task, ensure EliteDesk has the latest branch tip."

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Change #1** (handler field rename → grep e2e.rs): promote to `.claude/lessons/feedback_handler_field_rename_grep_e2e.md`
- [ ] **Change #2** (EliteDesk pre-queue fetch): update `/precheck` skill at `~/.claude/commands/precheck.md` — add "if bm-task, pull EliteDesk branch to HEAD" advisory line in Check 3 output
- [ ] **Change #3** (merge not rebase for phase-close conflicts): add to bm-merge brief template at `.claude/PRPs/templates/` or as a note in `.claude/commands/bm/bm-merge.md` Phase 2.2 DIRTY handling
- [ ] **Change #4** (adr-compliance fresh ack on merge-tip push): update `.claude/PRPs/reviews/pr-107-redflag-ack.md` pattern or add a step to bm-merge Phase 2.3

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
