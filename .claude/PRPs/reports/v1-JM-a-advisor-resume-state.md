# v1-JM-a advisor cold-resume brief (Task 9 → Task 10 boundary)

**Written**: 2026-04-23 at Task 9 commit close, impl session parked alongside (impl's own brief is at `C:/Users/barri/Developer/brehon-fork-phase-v1-JM-a/.claude/PRPs/reports/v1-JM-a-impl-resume-state.md`, untracked).
**Purpose**: Self-contained brief for a fresh advisor session to resume at the Task 9 → Task 10 boundary. Supersedes the Task 5→6 brief (overwrite precedent per AD-c).
**Resume when**: impl files the R10.1 DQ on next-session restart, OR impl hits any other decision point.

## TL;DR

- **Tasks 1–9 COMMITTED** on `phase-v1-JM-a`. Clean compile on `cargo check --workspace --features full` after every task. Task 8 reconciliation gate PASSED (27/27/27/27, empty diff). Task 9 Level-7 invariants PASS (32 defines, 32 re-exports, zero dup literals). Live DB parity proven via `config_parity_round_trip` (88 keys round-trip clean).
- **Phase branch**: `phase-v1-JM-a` @ `3537daa3b` (Task 9 tip). **NOT pushed to origin**.
- **Plan PR #91** (`plan/v1-JM-a` → `governance-v0`): still OPEN, CLEAN mergeStateStatus, no CR review yet. User hasn't asked advisor to poll.
- **DQ pending**: 0 at handover. **R10.1 will be filed by impl at next-session restart** (see §Immediate advisor action).
- **Three plan drifts caught**: R3.2 (Task 3), R5.1 (Task 5), R10.1 (Task 10 upcoming). All three need Task 11 retro entries.

## Cold-resume sequence

1. Read CLAUDE.md + `.claude/rules/*.md` (auto-loads in `-p` mode; in interactive mode, read manually)
2. Read this file in full
3. Read `.claude/PRPs/reports/v1-JM-a-advisor-brief.md` — operational playbook
4. Read `.claude/PRPs/reports/v1-JM-a-advisor-risk-register.md` — 14 entries, focus on R10.1 (line 193+)
5. Read `.claude/decision-queue.json` `pending` array — expect R10.1 to be there if impl restarted first, else 0
6. Read `.claude/runlog/bm-runlog.md` tail — last ~40 lines for BM-session state context
7. Verify state:
   ```bash
   git -C C:/Users/barri/Developer/brehon-fork-phase-v1-JM-a log --oneline 02189988d..HEAD
   # expect: at least 9 commits (plan cherry-pick + 8 task commits), possibly more if impl advanced
   git -C C:/Users/barri/Developer/brehon-fork-phase-v1-JM-a status --short
   # expect: just the untracked impl-resume-state.md, OR clean if impl re-committed it
   gh pr view 91 --repo barrie-cork/lemmy --json state,mergeStateStatus,reviewDecision
   # expect: OPEN unless user merged PR #91 during the park
   ```

## State at handover

### Branches + worktrees
- **primary worktree**: `C:/Users/barri/Developer/brehon-fork`, on `governance-v0` @ `8c85cd4ec` (4 commits ahead of `origin/governance-v0`; not pushed — 2 docs/advisor + 2 chore/bm setup + Task-5-boundary consolidation commits from 2026-04-23)
- **impl worktree**: `C:/Users/barri/Developer/brehon-fork-phase-v1-JM-a`, on `phase-v1-JM-a` @ `3537daa3b` (8 task commits + plan cherry-pick above trunk; not pushed)
- **plan branch**: `plan/v1-JM-a` @ `1bc32fcc0` on origin, PR #91 open

### Task-per-commit ledger (phase-v1-JM-a)
| Task | Commit | Status |
|---|---|---|
| plan | `92705f302` | cherry-picked from plan/v1-JM-a 2026-04-23 |
| Task 1 — Postgres enums | `2aa035a1a` | committed ✅ |
| Task 2 — columns + backfill | `9f1492858` | committed ✅ |
| Task 3 — Rust enums | `d9a1f25a5` | committed ✅ (interim-failure per R3.2) |
| Task 4 — schema.rs | `afb8c7a23` | committed ✅ (greens Task 3) |
| Task 5 — Diesel models | `7c46484e0` | committed ✅ (R5.1 GOTCHA-wording deviation) |
| Task 6 — config.rs (27 keys) | `7d4678c92` | committed ✅ (parity tests all pass) |
| Task 7 — seed migration + **Task 8 recon gate** | `c082ebb4c` | committed ✅ (27/27/27/27, empty diff; 88-key round-trip) |
| Task 9 — ENTRY_KIND consts | `3537daa3b` | committed ✅ (32 defines, 32 re-exports, 0 dup literals) |
| Task 10 — e2e round-trip + backfill smoke | — | **NEXT — BLOCKED on R10.1 DQ** |
| Task 11 — retro | — | pending |

### Plan-drift observations (ALL need Task 11 retro entries)

Detailed in risk register; abbreviated summary:

- **R3.2 — Task 3 validate `expect 0` is wrong** (drift vs Phase 1 precedent `083a9f3f9` which deliberately commits enums-only as interim failure). Resolution: impl committed with explicit fail-note per Phase 1 precedent; Task 4 greens.
- **R5.1 — Task 5 §10.7 GOTCHA wording incomplete** (claimed "Option<_> typing → no call-site change" but Rust struct literals require `..Default::default()`). Resolution: impl added `..Default::default()` to 1 in-scope (`create_report.rs:204`) + 1 OUT-list (`admin_emergency_remove.rs`, syntax-only) + ~10 e2e.rs sites. Commit message contains scope-deviation paragraph.
- **R10.1 — `PHASE_1_MIGRATION_COUNT = 9` shows v1-AD-a didn't extend** (drift NOT fixed by impl; plan §13 Task 10 sub-edit 1 line 1270 explicitly demands a DQ before picking the fix). Three options in the risk register R10.1 (line 193+). **Impl session parked at Task 9 will file this DQ on restart.** Advisor's first action post-resume is to answer it.

All three drifts are "plan-author mental model vs Rust/test-infra reality" class. Pattern-repetition hazard: JM-b/c/d/e + other v1 sub-phases will hit the same three wording gaps. **Plan-template fix recommended before v1-JM-b planning starts** (risk register retro carry-forward).

### DQ state
- `.claude/decision-queue.json` pending at handover: **0**
- Expected pending after impl resume: **1** (R10.1) — filed by impl as their first Task 10 action per plan §13 Task 10 line 1270

### PR state
| PR | Branch | Status | Notes |
|---|---|---|---|
| #91 | `plan/v1-JM-a` → `governance-v0` | OPEN, CLEAN mergeStateStatus, no CR review yet | BM session polls via `/bm-poll-cr 91` when user asks |

### Settings/tooling sanity
- Docker daemon: up for Task 7 e2e run (`config_parity_round_trip` passed)
- Wrapper probes 0/1/2/3/4: all passed at Task 0 (green from prior resume-brief)
- `.env` + `settings.local.json` in JM-a worktree: synced from primary 2026-04-23T19:45Z
- Telegram MCP: DISCONNECTED throughout; BM silent-skips pings per rules

## Immediate advisor action on resume — R10.1 DQ answer

**Expected DQ shape** (impl will file with `from: "impl"`, `answered_by: null`, options something like):

- (a) Extend by 3 (→12). Accept AD-a drift as pre-existing. File a separate DQ to retrofit AD-a in a later `chore(test): retrofit AD-a` commit.
- (b) Extend by 7 (→16). Fix drift inside JM-a Task 10. Scope-expansion risk.
- (c) Extend by 3 (→12) + inline TODO referencing AD-a drift + GH issue to track retrofit.

**Advisor lean (MED-HIGH): (c).**

Rationale:
1. **Scope discipline** — plan §13 Task 10 bumps from 9 → 12 (not 16). §18 risk row 7 anticipates AD-a drift and explicitly calls retrofit "someone else's problem". Plan §19 Notes "v1-AD-a precedent is the contract" means don't silently retroactively fix AD-a inside JM-a.
2. **Test is `#[ignore]`** — `phase1_migrations_round_trip` has `#[ignore = "TODO(v0-polish): deflake — GH issue #43 (needs revert-list extension for federation tables)"]`. The count mismatch doesn't block CI today, so the urgency to fix AD-a is low. BUT the revert-list MUST be right whenever someone un-ignores it.
3. **Inline TODO** — (c) leaves a breadcrumb at the exact file/line for the future un-ignore session. Better than (a) which buries the drift in a separate DQ.
4. **(b) is strictly worse than (c)** — adds 4 unrelated migrations to JM-a's test surface, multiplying debugging scope if anything breaks.

**However** — before committing to (c), check these two things:

1. **Verify the arithmetic.** Current count=9 with LIFO revert order means the 9 reverted migrations are (newest first):
   - 3 JM-a (2026-04-23 x3: enums, columns, seed) — NOT in current revert list, would be the "+3" bump
   - 4 AD-a (2026-04-22 x4: rule_set_versions, sponsor_allowlist, case_applied_config_snapshot, seed_v1_config_keys) — NOT in current revert list, would be the "+4" bump to reach 16
   - 1 federation_attestations (2026-04-21) — IS in current revert list
   - 2 governance_log_notify pair (2026-04-20 x2) — both IN current revert list
   - 1 restoration_sanction_variant (2026-04-19) — IN current revert list (Phase 5b Slice A)
   - 2 Phase 5a (2026-04-18 x2) — IN current revert list
   - 6 Phase 1 (2026-04-15 x6) — IN current revert list
   - **Total currently reverted = 12**, not 9. That contradicts the `PHASE_1_MIGRATION_COUNT = 9`.

   Hm, that's wrong. Let me recount what's IN the current 9: going LIFO from the boundary of "never applied until JM-a ran" (i.e. the trunk @ 02189988d HEAD just before phase branch), the top 9 migrations are:
   - federation_attestations (04-21) = 1
   - governance_log_notify pair (04-20 x2) = 3
   - restoration_sanction_variant (04-19) = 4
   - Phase 5a pair (04-18 x2) = 6
   - Phase 1 block (04-15 x6) = 12

   So 9 isn't clean LIFO from trunk — it matches the comment's arithmetic of "6 Phase 1 + 2 Phase 5a + 1 Phase 5b Slice A = 9" **but skips 4 migrations** (federation_attestations, log_notify pair). The comment at line 311-321 EXPLICITLY names the 9 as "6 Phase 1 + 2 Phase 5a + 1 Phase 5b Slice A" — i.e. the test was intended to revert exactly 9 specific migrations, but the runner reverts LIFO-by-count, so it currently reverts the wrong 9 (the top 9 after trunk HEAD, which at the time included federation_attestations etc., not what the comment claims).

   **This means R10.1 is bigger than plan §13 Task 10 anticipated.** The fix isn't just "extend by 3 or 7" — it's "the counting model is broken; whatever value we pick is a LIFO count, not a semantic-set count". The comment and the constant are out of sync even today.

   This changes the advisor answer. See §Revised lean below.

2. **GH issue #43** — cited in the `#[ignore]` attribute. Advisor should read the issue (if accessible) or at least acknowledge that whatever Task 10 does must be consistent with the issue's deflake scope.

## Revised advisor lean for R10.1 (POST-arithmetic-check)

Given the comment/constant mismatch: **do not fix the semantic model in JM-a.** Pick (c) with explicit acknowledgment in the TODO that the count is LIFO-positional and therefore whatever value Task 10 picks is an approximation.

**Proposed DQ answer** (advisor-hat, `docs(decision-queue)` subject):

> Pick (c): extend PHASE_1_MIGRATION_COUNT from 9 → 12 (JM-a's 3 additions only). Add inline TODO(v0-polish): reference GH issue #43 AND add a new GH issue sketch for "PHASE_1_MIGRATION_COUNT is a LIFO count, not a semantic set — comment claims it reverts specific named migrations but the runner reverts top-N-by-timestamp, so any post-trunk migration added after the last PHASE_1_MIGRATION_COUNT bump silently takes the Nth slot". Do NOT bump to 16 — AD-a retrofit is out-of-scope for JM-a per §18 risk row 7. Update the comment at e2e.rs:311-321 to reflect the real arithmetic (9 + 3 = 12, with JM-a's 3 migrations now on top of the LIFO stack).
>
> Citation: plan §13 Task 10 sub-edit 1 line 1270 ("file a DQ entry (blocking, answered_by: null) citing this §10 note and the discrepancy; do NOT self-resolve by silently extending"), plan §18 risk row 7, plan §19 Notes ("v1-AD-a precedent is the contract"), risk register R10.1 (line 193+).

Impl can then proceed with Task 10 using the +3 extension.

## Other anticipated advisor actions

### Task 10 backfill smoke test assertion count
Risk register R10.2 — 6 assertions (one per backfilled column) per plan §10.6 + §18 risk row 2. This is HIGH confidence, will answer immediately if impl queues it.

### Plan PR #91 ready to merge
If user pings "poll PR #91" or CR posts findings, BM hat runs `/bm-poll-cr 91`. If CR clean, ask user before merge. Post-merge, fast-forward `phase-v1-JM-a` to post-merge trunk — the plan cherry-pick `92705f302` dedupes against the identical patch on trunk; clean FF.

### Task 11 retro — plan-drift entries required
When impl writes the retro, confirm R3.2, R5.1, **and R10.1** entries all appear with root-cause + plan-amendment-recommendation sections. R10.1 carries a bonus action item: the count-model GH issue sketch.

### Handover skill retro (user request 2026-04-23)
At Task 11, add a dedicated section with design inputs for the future `/handover` skill — per memory `project_handover_skill_retro_pending.md`. Capture what worked (the resume-brief pattern, cold-read sequence, state-verification commands, attribution guardrails), friction points, and the two role flavors (advisor + impl).

## What this advisor does NOT resume into

- Running cargo (impl's lane; advisor verifies log tails if impl shares them)
- Git topology changes (BM hat only)
- Merging PR #91 (needs user confirm, BM hat)
- Writing on `phase-v1-JM-a` directly (impl's lane)
- Plan amendments on `plan/v1-JM-a` (that branch is behind PR #91 review; amendments after merge go to trunk or to future plans)

## Unresolved / parked

None load-bearing. Two cosmetic items left on primary worktree:
- `docs/brehon-law-inspired-network/Brehn-Consensus-*` — user's personal docs, untracked, unrelated
- `~$Brehn-Consensus-legal-brief.docx` — Word lockfile, worth gitignoring globally if Word docs become recurrent

## Contact surface at resume

- **User typing in-channel**: always the primary surface
- **Impl session**: running separately in `brehon-fork-phase-v1-JM-a`; contact via user-relay per `feedback_branch_manager_pm_split`. Impl communicates progress/DQs via git pushes on `phase-v1-JM-a` + user relay.
- **Telegram**: currently disconnected; ignore even if reconnects. DQ content never goes over Telegram per BM rules.

## Session-close ritual (for next advisor session-close)

Overwrite this file (AD-c precedent). Update the TL;DR, state-at-handover, anticipated-actions sections. Keep the cold-resume sequence skeleton stable.

---

**Written by**: advisor session 2026-04-23 at Task 9 commit `3537daa3b`
**Next advisor action trigger**: impl files R10.1 DQ on restart, or any unexpected impl signal
