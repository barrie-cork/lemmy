# Advisor handover — 2026-05-29 — quality-r2-task0-running

**Written:** 2026-05-29T19:58 UTC
**Author:** advisor session (C:\Users\barri\Developer\brehon-fork)
**Branch:** governance-v0 @ 03ea231bb
**Purpose:** Self-contained brief for next-session resume.

## TL;DR

- This session: shipped v1-rt-r3-followup gate-6 + transition + issue #163; then **kicked off v1-quality-r2** (active + re-scoped plan to 3-issue + clarify + bm-cut + Task 0 dispatch).
- Pending: **Junior Task #509 (v1-quality-r2 Task 0 pre-flight audit) is RUNNING** (started 19:47 UTC, cold cargo ~8-15 min). DQ pending = 0. No advisor-gate PRs.
- Blocked: nothing. Two recoverable incidents this session (stale-branch reuse on bm-cut; cross-session HEAD collision) — both RESOLVED, zero data loss.
- Next session should: **poll Task #509**, and on clean pass author + dispatch the T3 brief (14 fixtures-module doc-comment edits).
- External gate: Task #509 cold cargo running on the EliteDesk daemon (no CI; inline verification-only).

## Cold-resume sequence

1. Read `CLAUDE.md` + `.claude/rules/*.md` (auto-loaded in `-p` mode).
2. Read this file in full.
3. Read `.claude/decision-queue.json` — pending = 0 at handoff (re-check; a worker blocker DQ may have landed).
4. Read `.claude/runlog/bm-runlog.md` tail — last ~10 entries (the RT-r4 cut + collision-recovery notes are there).
5. Verify state:
   ```bash
   git fetch origin
   git rev-parse --short HEAD              # governance-v0 may have ADVANCED (concurrent RT-r4 session commits to trunk); was 03ea231bb
   git status --short                      # expect ?? .claude/PRPs/plans/v1-data-model-doc-and-04-retire.plan.md (RT-r4 session's, NOT yours — leave it)
   git worktree list                       # expect: brehon-fork [gov-v0], brehon-fork-redaction-r1, brehon-fork-rt-r4
   gh pr list --repo barrie-cork/lemmy --state open --json number,title,mergeStateStatus   # 2 dependabot PRs (#154, #151) — not advisor-gate
   ```
6. **Poll the in-flight task FIRST:** `mcp__junior-brehon__show_task` id=509. (Do NOT call `list_tasks status=done` — it overflows context; use show_task by id.)
7. Append cold-resume event to runlog:
   ```
   2026-05-29T<HH:MM>Z | advisor | meta | cold-resume | handover=advisor-2026-05-29-quality-r2-task0-running.md drift=<none|detail>
   ```

## State at handover

### Git
- Branch: `governance-v0`
- HEAD: `03ea231bb` (concurrent RT-r4 session is actively committing to trunk — expect this to have advanced on resume; that is NORMAL, not drift to worry about)
- Working tree: `?? .claude/PRPs/plans/v1-data-model-doc-and-04-retire.plan.md` (the RT-r4/data-model-retire session's untracked plan — NOT advisor-owned this session; leave alone)
- Unpushed: none (in sync with origin/governance-v0)

### Worktrees
- `C:/Users/barri/Developer/brehon-fork` [governance-v0] — canonical / this advisor session (v1-quality-r2 Mode B driving + meta-edits)
- `C:/Users/barri/Developer/brehon-fork-redaction-r1` [phase-v1-redaction-r1] — Lane A (redaction, concurrent, in-flight)
- `C:/Users/barri/Developer/brehon-fork-rt-r4` [phase-v1-RT-r4] — RT-r4 lane (MiniMax A/B trial ARMED; concurrent session; its OWN worktree now — created to resolve this session's shared-checkout collision)

### Open PRs
| PR# | title | branch | mergeState | reviewDecision |
|---|---|---|---|---|
| #154 | deps(cargo): bump cargo-all group | dependabot/cargo/... | UNKNOWN | none |
| #151 | deps(api_tests): bump npm-all group | dependabot/npm.../... | UNKNOWN | none |

Neither is advisor-gate (dependabot, no findings YAML, UNKNOWN mergeState is normal). Not this session's concern.

### Decision queue
- Pending requiring advisor: **none** (pending count = 0)
- Pending requiring impl: none
- Recently resolved this session: `a3d0e9941441-037` (advisor clarify pass on v1-quality-r2 re-scope — anchor counts 11/14/14 verified)

### Relays
- Advisor → impl awaiting response: none
- Impl → advisor awaiting answer: none

### Plan / PRD / ADR in-flight
- `?? .claude/PRPs/plans/v1-data-model-doc-and-04-retire.plan.md` — NOT advisor-authored this session (the concurrent data-model-retire/RT-r4 session's). Leave untouched.
- v1-quality-r2 plan (`.claude/PRPs/plans/v1-quality-r2.plan.md`) is COMMITTED + re-scoped (`74197312d` on gov-v0; present on phase-v1-quality-r2 @34dd093fc). Not in-flight.

## What's pending (ordered by priority)

1. **Poll Junior Task #509 (v1-quality-r2 Task 0 pre-flight audit).**
   - Context: dispatched 19:47 UTC, base `phase-v1-quality-r2`; cold cargo probes inline (~8-15 min); still RUNNING at handoff.
   - Command: `mcp__junior-brehon__show_task` id=509.
   - Expected outcome: `done` + NO commit on success. **VERIFY (never trust worker success):** all probes EXPECT satisfied, especially **Probe 7 anchor counts = EXACTLY 11 boot_context / 14 LEMMY_DATABASE_URL / 14 *_fixtures modules**, and **Probe 8 = no open PR touching e2e.rs**. Check for any `kind:blocker` DQ the worker raised (`git fetch origin phase-v1-quality-r2` + read DQ).
   - Rollback if it fails: on Probe 7 drift → read the blocker DQ, re-derive all T3/T4/T5 anchors from the actual e2e.rs file BEFORE any dispatch. On Probe 8 overlap (a redaction-r1 PR touching e2e.rs) → serialize/rebase before T3.

2. **On Task 0 clean pass → author + dispatch the T3 brief.**
   - Context: T3 = #156, the 14 fixtures-module `--test-threads=1` doc-comment header edits (first e2e.rs task). Plan §10.7 lists the 14 modules; §13 Task 3 is the spec.
   - File: `.claude/PRPs/briefs/v1-quality-r2-impl-3.md` (match plan Task-3 numbering; T1/T2 are SHIPPED-in-r2a so the gap is intentional). Author with **VERBATIM old_string/new_string anchors pulled LIVE from `crates/server/tests/e2e.rs` on the phase tip** (R11) — the plan's line numbers are STALE (post rt-r3-followup merge); read the actual `mod *_fixtures` opening lines. Canonical sibling: `.claude/PRPs/briefs/v1-quality-r2a-impl-0.md` (shape) + any rt-r3-followup impl brief (anchor discipline).
   - Then: commit to gov-v0 (atomic explicit-path protocol — see "What NOT to touch") → Mode B trunk→phase sync (daemon SSH merge gov-v0 into phase-v1-quality-r2) → dispatch `[role:impl-task]` base `phase-v1-quality-r2`.
   - Expected outcome: T3 worker edits 14 doc-comments, raises `validate-pending-laptop` (check+clippy+e2e-norun), advisor runs gates on laptop.
   - Rollback: if T3 anchors don't match the live file, re-pull the verbatim text and re-author the brief; do NOT dispatch with stale anchors.

3. **(After T3) → T4 (#159, EnvVarGuard hoist + boot_context refactor) → T5 (#160, 13 LEMMY_DATABASE_URL wraps) → T6 (retro).**
   - Context: serial (all e2e.rs, no `[P]`; T4 requires T3, T5 requires T4). **Author T4 anchors AFTER T3 lands** (T3's doc edits shift lines); author T5 anchors AFTER T4 lands (T4's hoist shifts lines). Do NOT author T4/T5 briefs now against the pre-edit tip.
   - Then: /brehon-verify (3 LIVE stories 3/4/5) → bm-pr → CR triage (gate 3) → merge-forward → merge (gate 5) → retro signoff (gate 6) → /brehon-phase-transition.

## Phase context (v1-quality-r2)

- **Scope:** the 3 remaining PR#155 carry-forwards NOT shipped in r2a — **#156** (fixtures `--test-threads=1` doc audit, 14 modules), **#159** (boot_context EnvVarGuard hoist+thread-guards), **#160** (13 LEMMY_DATABASE_URL setter sites → EnvVarGuard). All 3 OPEN.
- **r2a (PR #161) already shipped** #157 (DQ negative-duration lint) + #158-deferral DQ (`dd6012873857-001`; #158 stays OPEN). Gate-1 split the original 5-issue bundle → r2a (T1+T2) + r2 (T3+T4+T5, this).
- **Lane mode: B** (mobile remote-control) — all phase work via Junior; no laptop-side phase worktree; briefs authored on gov-v0 → trunk→phase SSH-sync.
- **Plan:** `.claude/PRPs/plans/v1-quality-r2.plan.md` — RE-SCOPED in place (banner at top; T1/T2 marked `[SHIPPED in r2a]`; live = T0/T3/T4/T5/T6).
- **Phase branch:** `phase-v1-quality-r2` @ `34dd093fc` (origin) — carries the re-scoped plan + Task 0 brief. Cut + FF-recovered (a stale r2a-era branch was reused by bm-cut #508; FF'd to gov-v0 HEAD).
- **Shape G SUSPENDED** through 2026-06-01 → cargo via validate-pending-laptop (advisor runs gates on THIS laptop). Task 0 is the exception (verification-only, cargo inline on daemon).
- **Clarify:** DQ `a3d0e9941441-037` verified the plan's 11/14/14 anchor counts hold on gov-v0; Task 0 Probe 7 re-verifies against the phase tip.

## Two resolved incidents this session (for retro)

1. **bm-cut #508 reused a STALE r2a-era `phase-v1-quality-r2` branch** (tip was an old PR#161 working-merge, 58 commits behind gov-v0). `git checkout -b` isn't idempotent against a pre-existing branch. FF-recovered (clean ancestor → fast-forward, not force). **Lesson candidate:** bm-cut brief should add a pre-cut "does phase-<X> already exist? delete-and-recut or FF-and-assert" step.
2. **Cross-session HEAD collision** — the concurrent RT-r4 session cut `phase-v1-RT-r4` and checked it out IN the shared canonical checkout while this session was mid-commit; my Task 0 brief commit landed on the wrong branch (local-only). Resolved: RT-r4 moved to its own worktree (`brehon-fork-rt-r4`); my brief re-landed byte-identical on `origin/phase-v1-quality-r2`. **Lesson candidate (reinforces multi-lane #5):** a new lane's bm-cut should be IMMEDIATELY followed by its own `git worktree add` (Mode A) — never share canonical.

## What NOT to touch

Per `.claude/rules/branch-manager.md`, `.claude/rules/handover.md`, `.claude/rules/multi-lane-worktree.md`:

- `crates/**`, `migrations/**`, `tests/**` — impl-owned (Junior workers edit these on the phase branch, not the advisor).
- `?? .claude/PRPs/plans/v1-data-model-doc-and-04-retire.plan.md` — the concurrent session's untracked plan. Leave alone.
- `brehon-fork-rt-r4` + `brehon-fork-redaction-r1` worktrees — other lanes' sessions. Never `git checkout` their branches in canonical; never write their DQ.
- **Atomic DQ/commit protocol on the shared canonical `.git/`** (multi-lane Hard refusal #6): concurrent sessions (RT-r4, data-model-retire) actively commit to gov-v0. For ANY gov-v0 commit: `git fetch → git status (confirm only YOUR file dirty) → git add <explicit path> → git diff --cached --name-only → git commit → git push`, as one uninterrupted sequence, and stage only your explicit path. NEVER `git add -A` / `git add .`.
- PR findings YAML at `.claude/PRPs/reviews/**` — BM-owned; read only.

## Bootstrap prompt (paste into next session)

```
I'm resuming advisor work on the Brehon governance fork. Previous session
wrote a handover at `.claude/PRPs/handovers/advisor-2026-05-29-quality-r2-task0-running.md`.

First steps:
1. Read CLAUDE.md + .claude/rules/*.md (auto-loaded in -p mode).
2. Read `.claude/PRPs/handovers/advisor-2026-05-29-quality-r2-task0-running.md` in full.
3. Execute its "Cold-resume sequence" in order (poll Task #509 FIRST).
4. Report the TL;DR summary plus any state drift you detected back to me.

Do not take any state-changing actions until I confirm.
```

## Closing state assertions (verify on resume)

- `git branch --show-current` → `governance-v0`
- `git rev-parse HEAD` → `03ea231bb...` OR LATER (concurrent sessions advance trunk — later is fine, not drift)
- `git status --short` → `?? .claude/PRPs/plans/v1-data-model-doc-and-04-retire.plan.md` (or empty if that session committed it)
- `.claude/decision-queue.json` pending count → 0 (unless Task #509 raised a blocker)
- `origin/phase-v1-quality-r2` → `34dd093fc` (or later if T3+ landed)
- Junior Task #509 → was `running`; expect `done` on resume — VERIFY probes per pending item #1
- Active handover file exists: `test -f .claude/PRPs/handovers/advisor-2026-05-29-quality-r2-task0-running.md`
