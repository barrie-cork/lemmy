# Bootstrap — v1-sponsor-liability-c (advisor session handoff)

**Author:** v1-SL-b advisor session, closing 2026-05-07T20:43Z
**Target:** Next advisor session that picks up v1-SL-c-1 (then v1-SL-c-2)
**Trunk HEAD at handoff:** `425d132ff` on `governance-v0`
**Last shipped:** PR #119 (v1-SL-b) merged at `9ae4c332c`

---

## §1. Role

You are the advisor for **Brehon v1-SL-c-1** — the first half of the v1-SL-c sub-phase split. v1-SL-c was split into c-1 + c-2 per DQ #150 (user override of planner's proceed-as-one lean DQ #148). c-1 ships the `sponsor_liability_grace` scheduler module + clokwerk wiring (no e2e); c-2 ships the 5 e2e behavioural tests.

This is a fresh session. Read the files listed in §2 before responding.

## §2. Files to read (in order)

1. `C:\Users\barri\Developer\brehon-fork\.claude\PRPs\plans\v1-sponsor-liability-c-1.plan.md` — the active plan (Shape G; Tasks 0, 1, 2, retro)
2. `C:\Users\barri\Developer\brehon-fork\.claude\PRPs\plans\v1-sponsor-liability-c-2.plan.md` — the sibling plan (run after c-1 ships)
3. `C:\Users\barri\Developer\brehon-fork\.claude\PRPs\briefs\sl-c-split-planning-1.md` — context for why c was split into c-1 + c-2
4. `C:\Users\barri\Developer\brehon-fork\.claude\PRPs\reports\session-retro-2026-05-07-sl-b-merge-close.md` — the v1-SL-b session retro with carry-forward lessons
5. `C:\Users\barri\Developer\brehon-fork\.claude\decision-queue.json` — note DQ #156 in pending (historical fail record from v1-SL-b e2e on tip 620861f08; NOT blocking, can be moved to resolved at retro time)
6. `C:\Users\barri\Developer\brehon-fork\CLAUDE.md` — confirm "active sub-phase" is updated to v1-SL-c-1
7. `C:\Users\barri\Developer\brehon-fork\.claude\rules\advisor-orchestrator.md` — operating rules (auto-loaded)

## §3. Git state at handoff

- **governance-v0 HEAD:** `425d132ff` (`docs(retro): session retro 2026-05-07 — v1-SL-b merge close`)
- **Phase branch HEAD:** `phase-v1-SL-c-1` does NOT exist yet — must be cut by `bm-cut` after plan approval
- **Recent governance-v0 commits:**

  ```
  425d132ff docs(retro): session retro 2026-05-07 — v1-SL-b merge close
  3462bd35f chore(advisor): post-merge bookkeeping — PR #119 merged 9ae4c332c
  9ae4c332c Merge pull request #119 from barrie-cork/phase-v1-SL-b
  363ac77e9 chore(adr): re-trigger adr-compliance scan with ack comment in place
  38294673b chore(merge): merge governance-v0 into phase-v1-SL-b — keep phase-branch DQ + PI_AUDIT_REPORT
  ```

## §4. First three actions for the incoming advisor

1. Run `/start-brehon v1-SL-c-1` to load live state (open PRs, DQ pending, Junior tasks).
2. Read `v1-sponsor-liability-c-1.plan.md` end-to-end. Run the **DoD smoke test** (every command in §15 literally) against current `governance-v0` HEAD per `feedback_pre_phase_dod_smoke_test.md`. Capture exit codes.
3. Run the **watchpoint specificity gate** per `feedback_advisor_watchpoint_specificity.md` — every watchpoint in §4 must cite a specific table, file, or `schema.rs` line. The plan has 14 watchpoints (carried verbatim from the trunk SL-c plan with watchpoint #12 annotated as c-2-only); confirm none are concept-only.

After both gates pass, surface to user for **plan approval** (mandatory user gate per CLAUDE.md). Do NOT queue bm-cut without explicit approval.

## §5. Decision-queue snapshot at handoff

```decision-queue-snapshot
pending:
  - id: 156
    from: advisor
    kind: validate-pending
    timestamp: 2026-05-07T15:30Z
    note: historical fail record for e2e tip 620861f08 (pre-fix-impl-4)
    blocking: NO — option-2 rule keeps fail records in pending[] for §G4 audit trail
    action: leave alone or move to resolved[] at v1-SL-b sub-phase retro time
```

## §6. Stop-and-ask tripwires (v1-SL-c-1 specific)

- **Stop and ask if:** `cargo check --workspace --features full` against current governance-v0 HEAD reports any error before bm-cut (workspace must be green at phase-cut time).
- **Stop and ask if:** the plan's §15 DoD references `cargo test --features full -p <crate>` — that flag combo is incompatible per `feedback_features_full_p_crate_incompatible.md`. Use `--workspace --features full` instead.
- **Stop and ask if:** v1-SL-c-1 plan's complexity score in §5.2 reads >8 — per `feedback_complexity_score_pre_split.md` the c-1 split was projected ~7; if it reads higher the planner should re-evaluate the split.
- **Stop and ask if:** the `sponsor_liability_grace` module file path in plan §11 differs from `crates/api/api/src/governance/sponsor_liability_grace.rs` (the brief's named path) — drift here means the plan or brief is stale.
- **Stop and ask if:** the `restoration-stub-only` watchpoint (#14, per DQ #145) appears to be re-litigated by the plan — restoration is stub-only in c-1 + c-2 + d, never fires; deviation requires a new ADR.

## §7. PR flow + process reminders

- **Phase-branch + PR flow.** After plan approval, queue `bm-cut` to create `phase-v1-SL-c-1` branched from `governance-v0`. All impl runs on the phase branch. Close the sub-phase via `bm-pr` opening PR `phase-v1-SL-c-1` → `governance-v0`. CodeRabbit auto-reviews.
- **Pre-phase harness audit.** Impl Task 0 runs the audit per `.claude/rules/pre-phase-harness-audit.md`.
- **DoD smoke test (advisor-side).** Advisor runs every plan §15 command literally before sign-off (see §4 above + `feedback_pre_phase_dod_smoke_test.md`).
- **Four-role orchestration.** v1-SL-c-1 runs under the four-role model. Advisor orchestrates Planning + Impl + BM as Junior subagents per `.claude/rules/advisor-orchestrator.md`. Polling cadence: ~10 min via `mcp__junior-brehon__list_tasks`. User gates: plan approval, judgment-heavy DQ, CR triage, merge confirm, retro sign-off.
- **Shape G.** v1-SL-c-1 ships under Shape G Layer G2 (push-and-exit). §15 references workflow YAMLs by path; cargo runs on GitHub-hosted runners. **Phase 2 e2e is NOT triggered for c-1** (c-1 ships zero new e2e tests). The e2e validation lives in c-2.
- **Cargo never runs on the EliteDesk worker.** Per `advisor-orchestrator.md` "Cargo never runs on the EliteDesk worker" — Shape G plans route cargo to GH Actions; pre-Shape-G plans route to laptop. v1-SL-c-1 is Shape G.

## §8. Carry-forward from v1-SL-b (high-leverage lessons)

These are the highest-value carry-forwards from the v1-SL-b session retro. Read the full retro at `.claude/PRPs/reports/session-retro-2026-05-07-sl-b-merge-close.md` for the rest.

1. **When a handler payload field is renamed, grep e2e.rs for the old field name BEFORE committing.** v1-SL-b cr-7b renamed `sponsor_pseudonym` → `revoker_pseudonym` in the handler but the brief scoped changes to `revoke_endorsement.rs` only — e2e assertions weren't updated, causing a Phase 2 e2e fail and a fix-impl-4 detour. Lesson candidate for promotion. Applies to c-1 if any payload field changes (unlikely — c-1 is module-only, but applicable to c-2 e2e tests).

2. **EliteDesk daemon doesn't auto-fetch before queueing.** Junior daemon branches from local HEAD on EliteDesk; if the laptop pushes a brief commit and immediately queues a task, the daemon may not have fetched yet, causing "brief not found" failures. Pattern: `ssh homeserver 'cd /srv/brehon-fork && git fetch origin <branch> && git merge --ff-only origin/<branch>'` before queueing any bm-task that references a freshly-pushed brief. This is the 4th occurrence of this family.

3. **`git merge --no-commit --no-ff` + `checkout --ours <conflicted-files>` for phase-close conflict resolution.** When phase branch and trunk diverge on meta-files (`decision-queue.json`, etc) but not code, one merge with two `--ours` resolutions beats 86 rebase rounds. Apply at v1-SL-c-1 close if PR #N shows `mergeStateStatus: DIRTY`.

4. **adr-compliance bypass may not satisfy the gate on a merge-tip push.** Even with prior `acknowledge` comments, the scan may fail on a fresh push. Workaround: post a fresh ack comment + empty re-trigger commit. Or — per user instruction at v1-SL-b close — merge directly past the advisory check when 0 fix-in-pr + recommendation=approve.

5. **Shape G two-phase validation works.** Phase 1 (workspace-check) catches compile/clippy issues; Phase 2 (e2e) catches behavioural drift. Don't compress these into a single gate. v1-SL-c-1 has Phase 1 only (no e2e); v1-SL-c-2 will run both.

## §9. Open follow-ups (not blocking c-1, surface at appropriate moment)

- **DQ #156 cleanup:** historical fail record in pending. Move to resolved[] at v1-SL-b sub-phase retro time (when that retro is eventually written).
- **v1-SL-b sub-phase retro:** the canonical `.claude/PRPs/retros/v1-SL-b-retro.md` was NOT written at handoff (user opted to skip per session-scope retro coverage). If that becomes important for weekly review or future audit, write it from `session-retro-2026-05-07-sl-b-merge-close.md` + the bm-runlog + the briefs.
- **MiniMax M2.7 A/B trial deferred:** per PMD memory `project_minimax_ab_trial_deferred.md`. Surface at next post-SL-b phase start (i.e., now, but only if the user proactively asks).
- **Lesson promotion candidates** from v1-SL-b session retro #1-#4 (handler field rename grep, EliteDesk pre-queue fetch, merge-vs-rebase, adr-compliance ack timing) — write to `.claude/lessons/feedback_*.md` if they recur in c-1 or c-2.

## §10. v1-SL-c-1 in one paragraph

v1-SL-c-1 ships the `sponsor_liability_grace` scheduler module per PRD §6 + §9.4 + §9.5 + §15 row 3 — the module + wiring half of the SL-c deliverable. New server-internal module at `crates/api/api/src/governance/sponsor_liability_grace.rs` exporting four public async functions (`run_grace_check_batch`, `evaluate_escape_conditions`, `fire_or_escape_case`, `check_grace_staleness`); a clokwerk tick block in `crates/routes/src/utils/scheduled_tasks.rs::setup` (sibling of the 15-minute `reputation_snapshot` block); a third atomic concurrency guard pair (`SPONSOR_LIABILITY_GRACE_RUNNING`). Three tasks (0 pre-flight + 1 module + 2 wiring) + retro. **No e2e tests in c-1** — those land in c-2. Projected complexity score: ~7.

---

_Generated by v1-SL-b closing advisor session 2026-05-07T20:43Z._
