# v1-JM-b advisor handover brief

**Written:** 2026-04-24 by the v1-JM-a dual-role (BM+advisor) session at JM-a close + JM-b kickoff
**Target session:** fresh advisor session for v1-JM-b — cold-resume from this brief
**Handover rationale:** v1-JM-a session context is approaching 200k tokens; JM-b benefits from a clean advisor context rather than inheriting stale session state

**Role scope:** this advisor **oversees JM-b impl**. Duties as defined in the JM-a advisor brief (§Purpose) — answer DQs, validate deviations, draft plan amendments. Impl runs in its own session on `brehon-fork-phase-v1-JM-a-b-impl` (or wherever the user cuts the impl worktree); this advisor runs in a fresh session (recommended: primary `brehon-fork` worktree on `governance-v0`, or a dedicated advisor worktree off trunk — user picks).

**Reads for cold resume (in order — 5-minute setup, 20-minute full load):**

**Immediate (must-read before any advisor action):**
1. `CLAUDE.md` + `.claude/rules/*.md` — auto-loads in `-p` mode; in interactive mode Read manually to surface phase-branch, branch-manager, decision-queue, pm-plugin-hooks-stable, pre-phase-harness-audit, cargo-output-capture
2. This file in full — handover state + what's new for JM-b
3. `.claude/PRPs/plans/v1-jury-mechanics-b.plan.md` — the plan this advisor supports. Read §1–4 for summary + lock-in decisions, §12 for OUT-of-scope, §13 for task breakdown, §18 for risks. Return to §10 patterns + §11 files-to-change on demand per DQ.
4. `.claude/PRPs/reports/v1-JM-a-advisor-brief.md` — **original working brief; structurally still valid for JM-b** with the amendments in this file's "What's new for JM-b" section. Re-read the §"Standard advisor responses lookup table" — unchanged for JM-b.

**JM-a context for pattern-mirroring (read fully — JM-b plan cites v1-AD-a + JM-a precedents throughout):**
5. `.claude/PRPs/reports/phase-v1-JM-a-retro.md` — the **authoritative source for active lessons.** Read §2 "What surprised" fully — the 4 R-items have become feedback-memory entries (see "What's new for JM-b" §1 below); §3 "What to carry forward" has plan-amendment items that apply to JM-b; §4 "What did NOT need fixing" is worth skimming so advisor doesn't re-introduce issues that were deliberately avoided.
6. `.claude/PRPs/reviews/pr-92-findings.yaml` — **16 findings, 15 done + 1 wont-fix.** Read the bucket:done entries; the findings body + addressing-commit map is the source-of-truth for "what did CR actually care about on JM-a" — JM-b advisor needs this so JM-b impl doesn't re-trip the same scope-contract finding patterns (e.g., ADR-015 pseudonymisation, protected-table-ALTER ADR-010 exception trails).
7. `.claude/PRPs/prds/v1-jury-mechanics.prd.md` — the PRD. Re-read §5.3 + §8.3 + §4.1 (the sections R5.2/R5.3 pointed at). These are the PRD references advisor relays MUST cite when naming enum values, constraint-cascade vocabulary, or `CaseStatusTier` variants.

**On-demand (read when triggered):**
8. `.claude/runlog/advisor-relays/*` (7 files from JM-a) — schema + examples. Re-read when advisor is about to author a new JM-b relay. `setup-relay-protocol.md` has the required frontmatter; `cr-9-enum-vocab-answer.md` is the canonical "how to cite PRD in an answer" example.
9. `.claude/runlog/bm-runlog.md` tail (~80 lines) — last BM-session state context. Primarily for BM-session handoff, but worth skimming to know what git state changed since merge.
10. `.claude/decision-queue.json` `pending` array — advisor's literal first action-target on every resume, once the above are loaded.

---

## TL;DR

v1-JM-a is **merged and closed** (PR #92 → commit `e1c22c759` on `governance-v0`). JM-b impl branch is **cut, plan-committed, and pushed** (`origin/phase-v1-JM-b` at `08ed5b1f9`). The advisor role stays the same as JM-a's brief described, with four new feedback-memory entries active in-session auto-load and one new retro-extraction system queued as a plan (`.claude/PRPs/plans/bm-retro-extract.plan.md`).

Immediate advisor action on cold-resume: **nothing urgent**. Impl session hasn't started JM-b yet; no DQ entries pending; no relays waiting. Advisor's first genuine duty is whenever impl files its first JM-b relay.

---

## State at handover

### Branches + PRs

- **PR #92 (v1-JM-a):** MERGED 2026-04-24T20:19:14Z via `--merge` (merge commit `e1c22c759`). 20-commit task-per-commit history preserved. Branch `phase-v1-JM-a` auto-deleted at origin; local branch retained in `brehon-fork-phase-v1-JM-a` worktree for retros.
- **JM-b impl branch:** `phase-v1-JM-b` at `08ed5b1f9` (one commit: `docs(plan): v1-JM-b — jury-mechanics sub-phase B implementation plan`, +1557 lines). Pushed to `origin/phase-v1-JM-b`.
- **JM-b worktree:** `C:/Users/barri/Developer/brehon-fork-phase-v1-JM-b` on `phase-v1-JM-b`. Submodules initialised; `settings.local.json` copied; ready for impl.
- **Plan worktree:** `C:/Users/barri/Developer/brehon-fork-plan-v1-JM-b` on `governance-v1`. Plan file still uncommitted there (was a planning sandbox; its value is fully extracted to `phase-v1-JM-b`). Safe to `git worktree remove` when user decides.
- **JMa worktree:** `C:/Users/barri/Developer/brehon-fork-phase-v1-JM-a` kept intentionally — holds unique impl-relays + retro artifacts not yet on trunk. User chose to leave it alone (see retro-archive gap below).

### Findings YAML

- `.claude/PRPs/reviews/pr-92-findings.yaml`: 16 findings, 15 done, 1 wont-fix, `recommendation: merged`. Gitignored; preserved locally for historical reference.
- `.claude/PRPs/reviews/pr-92-comment.md`: consolidated digest, posted at https://github.com/barrie-cork/lemmy/pull/92#issuecomment-4316018652.

### Decision queue

- `.claude/decision-queue.json`: verify on resume. Last known: 0 pending. No advisor action expected until impl writes.

### Advisor relays archived (from JM-a)

All 7 JM-a-era relays at `.claude/runlog/advisor-relays/`:
- `setup-relay-protocol.md` — schema this advisor uses (filename convention + frontmatter)
- `pr92-cr-findings.md` — first CR-fix triage
- `cr-9-enum-vocab-answer.md` — R5.2 correction
- `pr92-lows-batch.md` — lows batch greenlight
- `retro-tool-use-amendment.md` + `retro-cr-quality-amendment.md` + `retro-r53-enum-drift-pattern.md` — retro amendments

Impl-side relays (gitignored on JM-a worktree): 7 files including the cr-12 overreach RCA. Readable for archaeological value; not load-bearing for JM-b.

---

## What's unchanged from JM-a advisor brief

Everything in `.claude/PRPs/reports/v1-JM-a-advisor-brief.md` applies to JM-b advisor unless explicitly amended below. Key sections to re-read:

- **§Purpose** — the top-3 duties (answer DQs, validate deviations, draft plan amendments)
- **§Attribution discipline** — `answered_by: "advisor"` ONLY on `chore(advisor)` / `docs(decision-queue)` / `docs(advisor)` commit subjects
- **§Cold-read checklist** — same 6-step sequence (update: step 6 now points at PR-in-progress for JM-b, not JM-a)
- **§Standard advisor responses lookup table** — same DQ response patterns
- **§Test-substitution validation protocol** — same invariant-preservation check
- **§Plan amendment protocol** — same PRD-gate check (ADR violation = STOP and surface)
- **§What advisor does NOT do** — same out-of-scope list (no code under `crates/**`, no merge, no PR push)

---

## What's new for JM-b

### 1. Four new feedback-memory entries (auto-loaded in every session)

These were extracted from the JM-a retro §2 "What surprised" R-items and indexed under `MEMORY.md` section "Plan-authoring lessons (retro-derived)". Every JM-b session (impl or advisor) auto-loads these. **This advisor must verify the lessons are being applied**, especially at relay-draft time:

- **`feedback_advisor_cr_enum_drift.md`** — If a JM-b relay names enum values, column names, const names: MUST cite PRD §§line AND `file:line` cross-reference. Missing citation = relay is untrusted. Applies to advisor-authored relays too. **Two for two in JM-a — do not be the third.**
- **`feedback_insertform_default_propagation.md`** — If JM-b extends an `*InsertForm` (it will — jury-assignment write path + constraint-violation-log write path), the advisor reviewing the plan amendment or DQ answer must ensure `..Default::default()` propagation is named explicitly. "Option<_> auto-compat" is NOT correct.
- **`feedback_interim_failure_task_wording.md`** — If JM-b adds a Rust enum with `ExistingTypePath` in task N while `sql_types::X` lands in task N+1, the VALIDATE block must say "expect compile error" not "expect 0". If a DQ asks "plan says expect 0 but it fails — bug?": this is the answer.
- **`feedback_phase1_migration_count_lifo.md`** — JM-b is **handler-only per the plan** (no new migrations expected). Advisor MUST reject any impl ask that proposes "bumping `PHASE_1_MIGRATION_COUNT`" as part of a JM-b task — it's a plan-drift signal. If JM-b legitimately needs a migration, that's a plan amendment, not a silent bump.

### 2. New systemic gap identified — retro archiving

v1-JM-a exposed three artifact classes that die on worktree cleanup:
- `.claude/runlog/bm-runlog.md` (primary-only, not committed)
- `.claude/PRPs/reviews/pr-*-findings.yaml` + `pr-*-comment.md` (gitignored)
- `.claude/runlog/impl-relays/*` (untracked, phase-worktree-only)

A plan to fix this via `/bm-archive` is queued but not yet implemented. **For JM-b, the advisor should be aware** that any retro artifacts they produce face the same gap. Mitigation: commit advisor relays onto the phase branch explicitly (`git add .claude/runlog/advisor-relays/*.md; git commit -m "chore(advisor): JM-b relay <id>"`) rather than leaving untracked — this is the existing workaround until `/bm-archive` ships.

### 3. Queued system improvement — `/bm-retro-extract`

Plan at `.claude/PRPs/plans/bm-retro-extract.plan.md`. When impl finishes JM-b, advisor should consider prompting for this to land before JM-c — that way JM-b's retro lessons get auto-extracted instead of hand-written.

Advisor action on this: **mention it in the JM-b retro** if not yet implemented. Don't implement yourself — plan is BM-lane (changes `.claude/commands/bm/*` and rules).

### 4. Docker preflight enforcement (DQ #44 carry)

JM-a retro §4 confirms this is stable. No change for JM-b, but worth the advisor noting: if impl files a relay about e2e failing with testcontainers-related errors, first thing to ask impl to check is `docker ps > /dev/null 2>&1 && echo OK`. Docker Desktop stops on PC sleep/resume; this has bitten the project once.

### 5. Mid-phase events file — `v1-JM-b-retro-events.md`

`.claude/PRPs/reports/v1-JM-b-retro-events.md` is the durable archive for events the eventual `v1-JM-b-retro.md` will consume at phase close. Cold-resuming advisor should READ this file as part of resume — it captures patterns surfaced after this handover was written. As of last advisor-session write, 4 events are captured:

1. **Event 1** — Task 0 clippy baseline check skipped → debt discovered mid-Task-4 (the `ConfigScope::Community` unfulfilled `#[expect(dead_code)]` from v1-AD era, plus 9 others)
2. **Event 2** — JM-a R5.1 lesson incompletely transferred to JM-b drift-fix `cb7b6bdb2` (R5.1 fired again; `..Default::default()` claim was false for explicit-field test callers)
3. **Event 3** — Plan §13 Task 4/5 vs Task 9 + DoD clippy-flag inconsistency (`--no-deps` only on Task 9 + DoD, not on per-task checkpoints)
4. **Event 4** — META: gate-keeper Task 0 steps skipped under plan-write time pressure (Event 1 + Event 2 + JM-a R10.1 share this shape; pattern-level fix recommended)

Each event has a documented plan-amendment recommendation for JM-c/d/e and a memory-note recommendation. Advisor responsibility: append new events as they fire during JM-b impl; do NOT preemptively merge into the retro (let phase-close author do that).

**Where it lives:** primary worktree (`brehon-fork`) at `.claude/PRPs/reports/v1-JM-b-retro-events.md`. Rationale: advisor-lane artifact, primary stays on `governance-v0`, no need to commit to `phase-v1-JM-b` until the retro phase. (Same artifact-class as the JM-a era impl-relays that lived gitignored on the JM-a worktree.)

---

## Immediate advisor action on cold-resume

**Gate role:** this advisor is **JM-b impl oversight**. That means: answer impl's DQs + relays, validate deviations vs plan + PRD, catch the R5.2/R5.3-class enum-drift before impl writes code, draft plan amendments when PRD-faithful but plan-inaccurate.

**Queue state on arrival:** empty (0 DQs, 0 pending impl-relays). Impl hasn't started JM-b yet. First genuine duty: whenever impl files its first JM-b relay under `.claude/runlog/impl-relays/` (on the `brehon-fork-phase-v1-JM-b` worktree), user relays it to advisor; advisor responds per the standard response protocols in the original JM-a brief + the new feedback-memory lessons.

Likely first-relay topics for JM-b (best guess, not guaranteed):
- Cascade-helper signature (`get_int_cascade` / `get_float_cascade`) — first consumer is in JM-b per JM-a handoff notes §3.5
- `jury_constraint_violation_log` write construction — ensuring `reason_code` uses the 4-value enum (not free text) per ADR-015 + R5.2 lesson
- Eligibility query shape when `jury_age_requirement_days = 0` (test-fixture pattern)

All three have pre-established PRD sections (§5.3, §8.3, §4.1) — advisor answers should cite them verbatim per the enum-drift lesson.

---

## Cold-resume verification sequence (post-handover)

```bash
# From primary worktree
cd C:/Users/barri/Developer/brehon-fork
git fetch origin --prune
git log --oneline -5 origin/governance-v0
# expect: top is e1c22c759 "Merge pull request #92 from barrie-cork/phase-v1-JM-a"

git worktree list
# expect: 3 active worktrees (primary on governance-v0, JM-a on phase-v1-JM-a, JM-b on phase-v1-JM-b)
# Plan worktree (brehon-fork-plan-v1-JM-b) may still be present or removed — both fine

# Check JM-b branch state
cd C:/Users/barri/Developer/brehon-fork-phase-v1-JM-b
git log --oneline -3
# expect: top is 08ed5b1f9 "docs(plan): v1-JM-b — jury-mechanics sub-phase B implementation plan"
# parent is e1c22c759 (the JM-a merge)

ls .claude/PRPs/plans/v1-jury-mechanics-b.plan.md
# expect: 109823 bytes

# Verify PR state (merged) + no new PRs opened
cd C:/Users/barri/Developer/brehon-fork
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,baseRefName
# expect: empty list (JM-b PR not opened yet — PR is cut at phase-close, not at phase-start)

# Verify findings YAML is preserved (JM-a history)
ls .claude/PRPs/reviews/pr-92-findings.yaml
# expect: file exists, ~14-16KB

# Decision queue check
cat .claude/decision-queue.json | python -c "import sys, json; d=json.load(sys.stdin); print('pending:', len(d.get('pending', [])))"
# expect: pending: 0

# Memory check — new lessons loaded
ls ~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/feedback_advisor_cr_enum_drift.md
# expect: file exists
```

If any step fails or produces unexpected output: STOP, do not proceed to advisor duties until the state is reconciled. Re-read this brief + the JM-a advisor brief + ask user to confirm state before making any commits.

---

## What this advisor should NOT carry over from the dual-role JM-a session

The JM-a session wore **BM + advisor** hats in one primary-worktree process. That was necessary during JM-a's PR lifecycle because BM needed continuous state for poll-cr / triage / merge runs. **For JM-b, split the roles**: BM stays in this (or another) primary-worktree session, advisor becomes its own session with this brief as cold-state.

Reason: the dual-role session is carrying ~15+ sub-sessions of Agent-delegated BM work in its history. Starting fresh for advisor keeps the context window under 200k through the entire JM-b cycle, which the dual-role JM-a session approached by the merge point.

If cold-resume reveals the JM-b cycle also needs dual-role, that's fine — revert to the JM-a pattern explicitly and update the role-topology line in `.claude/runlog/bm-runlog.md`.

---

## Session-close ritual (same as JM-a)

At the end of every advisor-session work segment, write a cold-resume brief to `.claude/PRPs/reports/v1-JM-b-advisor-resume-state.md` (overwrite pattern from v1-AD-c / v1-JM-a precedent). The brief must state:

- Current trunk HEAD + phase branch HEAD
- JM-b PR state (not-yet-opened / open / merged)
- Open DQ entries with advisor-lean recorded but unanswered
- Any in-flight plan amendments (branch, PR, status)
- Anticipated next advisor action at resume

This is what let AD-c and JM-a resume cleanly after overnight breaks.
