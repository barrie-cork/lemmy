# Advisor handover — 2026-04-25 — jm-c-impl-oversight

**Written:** 2026-04-25T15:30Z
**Author:** advisor session (`C:\Users\barri\Developer\brehon-fork`)
**Branch:** governance-v0 @ `4347284e0`
**Purpose:** Self-contained brief for the next advisor session that will oversee v1-JM-c impl.

## TL;DR

- This session: cold-resumed for JM-c (post-PR-#95 merge); authored the 1738-line JM-c plan; cut `phase-v1-JM-c` + worktree via BM; opened + merged plan PR #97 (`73c208f2b` on trunk); landed `chore(prp): principle-style skill triggers for prp-implement + prp-plan` directly on `governance-v0` (`4347284e0`); pushed; saved `feedback_principles_not_rules.md` memory entry.
- Pending: 1 non-blocking DQ (#47, planner-attributed, v1.5 territory); 0 advisor-gate PRs (PR #94 `chore/prp-plan-phase5-opus-opt-in` is in trunk, not advisor-gate); 0 advisor-lane plan/PRD edits in flight; 0 advisor relays awaiting impl response. JM-c impl session has NOT yet started — `phase-v1-JM-c` worktree is cut and provisioned but no impl commits exist.
- Blocked: nothing on advisor side. JM-c impl is the next actor; advisor's role is purely reactive (DQ answers, deviation validation, relay responses) until impl starts work.
- Next session: at minimum, run cold-resume sequence + verify state matches "Closing state assertions" below. If impl has started JM-c work in the meantime, prioritise reading `.claude/runlog/impl-relays/` and the JM-c worktree's most recent commits before any advisor action.
- External gates: none active. PR #97 (plan) merged. PR #94 (chore/prp-plan-phase5-opus-opt-in) is OPEN with mergeStateStatus=UNKNOWN — not authored by this session, status check pending; flag for triage but not advisor-gate.

## Cold-resume sequence

1. Read `CLAUDE.md` + `.claude/rules/*.md` (auto-loaded in `-p` mode; in interactive mode Read manually: `branch-manager.md`, `decision-queue.md`, `handover.md`, `phase-branch.md`, `pre-phase-harness-audit.md`, `pm-plugin-hooks-stable.md`, `cargo-output-capture.md`, `no-cargo-output-paste.md`, `governance-log-entry-kind-registry.md`).
2. Read this file in full.
3. Read `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` (the plan this session authored; 1738 lines on trunk @ `4347284e0`). Read §§1-7, §10, §13, §17-§19 for advisor-relevant context. Defer §10 patterns + §13 task bodies to demand reads when impl files relays.
4. Read `.claude/PRPs/reports/v1-JM-b-advisor-handover.md` — predecessor JM-b advisor brief (still structurally valid for JM-c); read §"What's unchanged from JM-a advisor brief" + §"What's new for JM-b" — the 4 feedback-memory entries flagged there ALSO apply to JM-c.
5. Read `.claude/PRPs/reports/v1-JM-b-retro.md` §3.3 (handoff notes for JM-c — verbatim).
6. Read `.claude/PRPs/reports/v1-JM-b-retro-events.md` Events 1-4 — the 7 plan-template amendments rolled into JM-c plan §10 + §13. Verify the JM-c plan reflects them by skimming §10.2 (R1 i64::from), §10.7 (R2 reputation_snapshot seeding), §13 Task 0 (R5 enumerated probes).
7. Read most recent `.claude/runlog/advisor-relays/*` — the only JM-c-era relay would be a future one (none yet); the latest is `pr95-cr-3-emergency-remove-cascade.md` from PR #95 close (already addressed).
8. Verify state:
   ```bash
   git fetch origin
   git rev-parse HEAD              # expect 4347284e0
   git status --short              # expect 1 M (bm-runlog) + 7 untracked legal-brief docs
   gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,mergeStateStatus
   # expect: PR #94 still open (or merged); no JM-c PR yet (impl creates that at phase close)
   cat .claude/decision-queue.json | python -c "import sys, json; print('pending:', len(json.load(sys.stdin).get('pending', [])))"
   # expect: pending: 1
   ```
9. Append cold-resume event to runlog (per `.claude/rules/handover.md`):
   ```
   <ISO-UTC> | advisor | meta | cold-resume | handover=advisor-2026-04-25-jm-c-impl-oversight.md drift=<none|<detail>>
   ```

## State at handover

### Git
- Branch: `governance-v0`
- HEAD: `4347284e0` (`chore(prp): principle-style skill triggers for prp-implement + prp-plan`)
- Working tree:
  - `M .claude/runlog/bm-runlog.md` (BM's pending appends from JM-c plan-cycle; not advisor-owned, do NOT touch)
  - `?? docs/brehon-law-inspired-network/Brehn-Consensus-{legal-brief.docx, two-part-explainer.md, .pdf}` + `expert-review-suite/` + `~$ehn-Consensus-legal-brief.docx` + `~WRL{1116,2369}.tmp` (advisor-lane legal-brief work in progress; NOT this session's; do not commit until user signals ready; the `~$` and `~WRL*.tmp` are Word scratch files that should never be committed)
- Unpushed: clean (origin/governance-v0 == 4347284e0 == HEAD)

### Worktrees
| Path | Branch | HEAD | Purpose |
|---|---|---|---|
| `C:/Users/barri/Developer/brehon-fork` | `governance-v0` | `4347284e0` | Primary — advisor lane (this brief written here) |
| `C:/Users/barri/Developer/brehon-fork-phase-v1-JM-a` | `phase-v1-JM-a` | `92950055f` | Retained for JM-a retro reads (JM-a merged at PR #92) |
| `C:/Users/barri/Developer/brehon-fork-phase-v1-JM-b` | `phase-v1-JM-b` | `918b1f872` | Retained for JM-b retro reads (JM-b merged at PR #95) |
| `C:/Users/barri/Developer/brehon-fork-phase-v1-JM-c` | `phase-v1-JM-c` | `9e5dd60a5` | **JM-c impl worktree — preserved post-merge for impl session** |
| `C:/Users/barri/Developer/brehon-fork-plan-v1-JM-b` | `governance-v1` | `563d7904c` | JM-b ephemeral plan sandbox; safe to remove |

### Open PRs
| PR | Title | Branch | Base | mergeState | reviewDecision |
|---|---|---|---|---|---|
| #94 | chore(prp-plan): Phase 5 HIGH-complexity parallel-Explore opt-in | `chore/prp-plan-phase5-opus-opt-in` | `governance-v0` | UNKNOWN | (none) |

PR #94 is **NOT** advisor-gate by the bm-runlog or any session-written context — it appears to be a chore PR authored outside the JM advisor sessions (possibly from CC config upgrade work). It is open with `mergeStateStatus: UNKNOWN`. Worth a triage glance but not blocking JM-c oversight. Recommend: read `gh pr view 94 --repo barrie-cork/lemmy` at session start to determine authorship + if it touches `prp-plan.md` (which this session just edited at `4347284e0`) — if so, there's potential for merge conflict that the next session should flag.

### Decision queue

- **Pending requiring advisor:** 0
- **Pending requiring impl (impl-self-resolvable / planner-attributed):** 1
  - **#47** (planner) — OQ-V1-JM-07 — Post-JM-b general case-open severity-tier inference. JM-c plan §12 + §19 explicitly defer; v1.5 territory; **does NOT block JM-c**. Lean documented in plan body (option a — hardcoded reason_code → severity_tier table). Impl session does NOT need to resolve.
- **Recently resolved (since 2026-04-24 handover):**
  - #44 (Docker preflight — answered_by user, applied to pre-phase-harness-audit.md §0)
  - #46 (v1/limitation GH issue capture — answered_by user, applied to /prp-implement Phase 5 REPORT template)
  - #49 (JM-b clippy debt strategy — answered_by impl-self-resolved at JM-b Task 4)

### Relays
- **Advisor → impl awaiting impl response:** 0
- **Impl → advisor awaiting advisor answer:** 0
- All JM-a / JM-b era relays resolved by their phase merges. JM-c-era relays will appear when impl begins; the file location convention is `.claude/runlog/impl-relays/<id>.md` (impl-side, in JM-c worktree) and `.claude/runlog/advisor-relays/<id>.md` (advisor-side, in primary worktree).

### Plan / PRD / ADR in-flight
- 0 modified files under `.claude/PRPs/plans/`, `.claude/PRPs/prds/`, `docs/brehon-law-inspired-network/` (other than the untracked legal-brief work which is NOT this session's scope).
- The JM-c plan landed on trunk via PR #97; no in-flight edits remain.

## What's pending (ordered by priority)

1. **Watch for JM-c impl-relay activity.**
   - Context: impl session has not started yet; when it does, it will write relays to `.claude/runlog/impl-relays/` on the JM-c worktree (`C:/Users/barri/Developer/brehon-fork-phase-v1-JM-c`). Advisor reads those + responds via `.claude/runlog/advisor-relays/*.md` on primary.
   - File/command: `ls -la C:/Users/barri/Developer/brehon-fork-phase-v1-JM-c/.claude/runlog/impl-relays/` (compare to current state: 1 file, `pr92-cr-12-overreach-revert-ask.md`, JM-a era — anything new is JM-c)
   - Expected outcome: zero new files = impl hasn't started; ≥1 new file = impl is mid-phase, advisor reads + responds.
   - Rollback if it fails: N/A — purely reactive.

2. **Triage PR #94 if it's still open.**
   - Context: appeared during this session's `git fetch`; not authored by this session; mergeStateStatus UNKNOWN. Title suggests it edits `prp-plan.md` — same file this session edited at `4347284e0`. Potential merge conflict.
   - File/command: `gh pr view 94 --repo barrie-cork/lemmy --json author,createdAt,headRefName,files,mergeable,mergeStateStatus,body`
   - Expected outcome: identify whether PR #94 conflicts with the `chore(prp): principle-style skill triggers` commit at `4347284e0`. If yes, decide whether to (a) close PR #94 (its scope subsumed by `4347284e0`), (b) request rebase, (c) leave alone if the edits are orthogonal.
   - Rollback if it fails: N/A — read-only triage.

3. **Verify JM-c plan §10/§13 amendments R1-R7 actually landed in the committed plan.**
   - Context: this session authored the plan with all 7 JM-b retro amendments rolled in, but the impl session will be reading the as-committed version (1738 lines on trunk). A defensive cross-check before impl starts costs minutes and prevents an Event-4-style amendment-drop surprise mid-phase.
   - File/command: `grep -n "R1\|R2\|R3\|R4\|R5\|R6\|R7\|i64::from\|seed_jury_eligible_snapshots\|enumerate ALL\|--no-deps\|cargo test --no-run" .claude/PRPs/plans/v1-jury-mechanics-c.plan.md | head -30`
   - Expected outcome: each of R1-R7 has multiple grep hits; if any R-id is absent or thin, advisor amends the plan via a `docs(plan): v1-JM-c — clarify R<N> amendment` commit on trunk (advisor-lane chore is acceptable on trunk per `chore(ops)` precedent).
   - Rollback if it fails: revert the amendment commit; impl session can start without it (R-amendments are advisory).

4. **(Standing reactive duty)** Answer impl-side DQ entries when they appear, applying the standard advisor protocols from JM-a brief §"Standard advisor responses lookup table" + the 4 active feedback-memory entries (`feedback_advisor_cr_enum_drift.md`, `feedback_insertform_default_propagation.md`, `feedback_interim_failure_task_wording.md`, `feedback_phase1_migration_count_lifo.md`).

5. **(Standing reactive duty)** When CR posts findings on the eventual JM-c PR, BM session triages per `feedback_pr_review_triage_pattern.md` (4-bucket: fix-in-pr / rebut / carry-forward / done); advisor reviews BM's triage proposals and validates against PRD + ADRs before BM posts the consolidated comment.

## What NOT to touch

Per `.claude/rules/branch-manager.md` and `.claude/rules/handover.md`:

- **`crates/**`, `migrations/**`, `tests/**`** — impl-owned. Advisor session NEVER edits these.
- **`.claude/PRPs/plans/v1-jury-mechanics-c.plan.md`** — plan is committed at `4347284e0`. Do NOT silently edit. If amendments are needed (Item 3 above flagging an R-amendment drop), use `docs(plan):` commit on trunk with the amendment justified inline.
- **`.claude/PRPs/reviews/pr-*-findings.yaml`** — BM-owned; read only.
- **`.claude/runlog/bm-runlog.md`** — BM-owned (currently has unstaged appends from JM-c plan cycle pending bundle commit).
- **`.claude/decision-queue.json`** — only write `answered_by: "advisor"` in commits with subject `chore(advisor):` or `docs(decision-queue):` per `.claude/rules/decision-queue.md` §"Attribution integrity". Per `feedback_principles_not_rules.md` (saved this session): never use the advisor label opportunistically.
- **`.claude/PRPs/handovers/`** — this session's brief; future advisor sessions write their OWN brief, do not edit this one.
- **Untracked `docs/brehon-law-inspired-network/Brehn-Consensus-*` files + `expert-review-suite/` + `~$*.docx` + `~WRL*.tmp`** — user-authored legal-brief work in progress; advisor leaves alone until user signals.

## Bootstrap prompt (paste into next session)

```
I'm resuming advisor work on the Brehon governance fork. Previous session
wrote a handover at `.claude/PRPs/handovers/advisor-2026-04-25-jm-c-impl-oversight.md`.

First steps:
1. Read CLAUDE.md + .claude/rules/*.md (auto-loaded in -p mode).
2. Read `.claude/PRPs/handovers/advisor-2026-04-25-jm-c-impl-oversight.md` in full.
3. Execute its "Cold-resume sequence" in order.
4. Report the TL;DR summary plus any state drift you detected back to me.

Do not take any state-changing actions until I confirm.
```

## Closing state assertions (verify on resume)

- `git branch --show-current` → `governance-v0`
- `git rev-parse HEAD` → `4347284e0b3153896d2a89b29ae1f935f809bb9b`
- `git rev-parse origin/governance-v0` → `4347284e0b3153896d2a89b29ae1f935f809bb9b` (clean; pushed)
- `git status --short` → exactly:
  - `M .claude/runlog/bm-runlog.md`
  - 7 untracked under `docs/brehon-law-inspired-network/` (Brehn-Consensus-* + expert-review-suite/ + ~$ + ~WRL*.tmp)
- `.claude/decision-queue.json` pending count → 1 (DQ #47, planner-attributed, non-blocking)
- Open PRs:
  - PR #94 (chore/prp-plan-phase5-opus-opt-in) — open, NOT advisor-gate; triage per Pending Item 2
  - No JM-c PR (impl creates that at phase close, not phase start)
- Active handover file exists: `test -f .claude/PRPs/handovers/advisor-2026-04-25-jm-c-impl-oversight.md`
- JM-c plan on trunk: `test -f .claude/PRPs/plans/v1-jury-mechanics-c.plan.md` AND first line is `# Plan: v1-JM-c — submit_jury_vote 9-step handler (snapshot-aware threshold + deadlock + appeal_window)`
- JM-c worktree intact: `test -d C:/Users/barri/Developer/brehon-fork-phase-v1-JM-c` AND `git -C C:/Users/barri/Developer/brehon-fork-phase-v1-JM-c rev-parse HEAD` → `9e5dd60a5...`
- Memory entries this session added:
  - `~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/project_v1_JM_c_plan_written.md`
  - `~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/feedback_principles_not_rules.md`

---

## Notes for next advisor session — don't re-litigate

- **Skill triggers in prp-implement / prp-plan are committed at `4347284e0`** (this session, principle-style framing per `feedback_principles_not_rules.md`). The triggers are advisory — they tell the impl session WHEN the three skills (`/cargo-validate`, `/test-write`, `/edit-mechanical`) shorten the work, with explicit "inline is the right shape when…" guardrails. Do NOT add stronger mandates; the user explicitly chose the conservative framing.
- **JM-c is the cross-PRD gate** for JM-d (reads `appeal_window_expires_at`), SL-d (sponsor-liability compute graft at step 7 TODO), rep-tuning-r3 (vote-outcome emitters at step 7). When CR or impl raises scope questions, the answer is: stay in JM-c lane; do NOT silently subsume issue #96 carry-forward findings (cr-5 / cr-8 in particular).
- **DQ #47 is planner-attributed and stays pending** until v1.5 planning. Do NOT answer it; impl does NOT need it for JM-c.
- **The v0 literal `"jury_vote_submitted"` at `submit_jury_vote.rs:191` deliberately stays.** Cleanup is a separate carry-forward (not in JM-c scope, not in issue #96). If CR flags it on the JM-c PR, response is "out of scope; v0 literal preserved for log-consumer compatibility; cleanup is a separate `chore(governance-log)` PR with backfill UPDATE."

_Handover author: advisor session 2026-04-25T15:30Z. Scope: JM-c plan authored + plan PR #97 merged + skill-trigger chore committed + memory written. Next advisor: reactive oversight of JM-c impl when it begins._
