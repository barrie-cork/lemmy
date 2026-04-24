# v1-JM-a advisor working brief

**Written**: 2026-04-23 by advisor (BM+advisor dual-role session) at JM-a impl kickoff
**Session topology**: option (a) — BM+advisor dual-role in primary worktree `brehon-fork` on `governance-v0`. Impl runs separately in `brehon-fork-phase-v1-JM-a`.
**Plan**: `.claude/PRPs/plans/phase-v1-JM-a.plan.md` (1558 lines, 11 tasks + Task 0 pre-flight)
**PRD**: `.claude/PRPs/prds/v1-jury-mechanics.prd.md` §17 row 1

## Purpose

Operational playbook for the advisor half of the dual-role session during v1-JM-a impl. Top-3 duties per user confirmation 2026-04-23:

1. **Answer blocking DQ entries** — impl writes, advisor answers with `answered_by: "advisor"`, committed via `docs(decision-queue)` subject
2. **Validate plan deviations** — impl flags test substitution / scope creep / unexpected state; advisor validates against PRD + plan invariants before impl commits
3. **Plan amendments** — if impl discovers plan is wrong mid-flight, advisor patches the plan file on a separate branch + PRs back to trunk

Duties explicitly OUT-OF-SCOPE for this advisor (kept separate):
- Running cargo / writing code (impl's lane)
- Git topology (BM's lane — same session but different artifact class)
- Rebuttal drafting on CodeRabbit findings (BM lane via `/bm-triage`)
- Writing or editing code under `crates/**` (BM rule hard-block)

## Attribution discipline (load-bearing)

Per `.claude/rules/decision-queue.md:73-97`:

- This session MAY write `answered_by: "advisor"` ONLY when the commit subject is `chore(advisor)` or `docs(decision-queue)` or `docs(advisor)`.
- ANY other commit subject (`feat(...)`, `chore(bm)`, `docs(plan)`) that introduces an `answered_by: "advisor"` entry is a **process breach** requiring a `docs(attribution)` follow-up commit.
- **Pre-seeded answers from planning** (if any pre-existed in the plan file) are `answered_by: "planner"`, not `"advisor"`.
- If this session is ever uncertain whether it is "wearing the advisor hat" or "wearing the BM hat" for a given action: the attribution rule decides. BM actions never touch `answered_by: "advisor"`. Advisor DQ-answer commits never touch PR state.

## Cold-read checklist for advisor resume

When resuming this advisor role in a fresh session (e.g. cold-resume next morning), read in this order:

1. `CLAUDE.md` + `.claude/rules/*.md` (auto-loads in `-p` mode; in interactive mode read manually)
2. This file in full
3. `.claude/PRPs/reports/v1-JM-a-advisor-risk-register.md` — risk register + likely decision points
4. `.claude/decision-queue.json` `pending` array — any unanswered questions (advisor's first action at resume)
5. `.claude/runlog/bm-runlog.md` tail — last ~40 lines for BM-session state context
6. If impl has pushed `phase-v1-JM-a` since last resume: `git log 02189988d..origin/phase-v1-JM-a --oneline` + `gh pr view 91` for plan PR state

## Standard advisor responses — lookup table

### When impl writes a DQ entry

| DQ pattern | Advisor action |
|---|---|
| "Plan says X, code has Y — which is right?" | Read both, check PRD §N it cites, check v1-AD-a precedent (plan §19 Notes: "v1-AD-a precedent is the contract"), pick + write answer with code/line citation |
| "Found pre-existing upstream debt blocking my DoD command" | Two paths: (1) narrow DoD in plan amendment, or (2) clear debt in pre-task chore commit. Prefer (2) if <30 LoC; else (1) with evidence. |
| "Task N produces different output than plan expected — advisor wants a swap / rewrite?" | Check if the plan invariant still holds; if yes, greenlight divergence in DQ answer with "plan §M still satisfied via Y"; if no, escalate to plan amendment |
| "Blocked by something outside my scope (BM, CR, external)" | Acknowledge + park + redirect impl to self-resolve OR defer to BM session ping |
| "Is this an advisor-authority decision or an impl judgment call?" | Apply DQ rule §3 test: does the answer gate all remaining work? If yes → advisor authoritative; if no → impl can self-resolve with evidence |

### When impl flags a test substitution

Per DQ #45 resolution (v1-AD-d retro), the rule is: **substitutions that DROP any plan-named invariant MUST be queued to advisor BEFORE commit**. Coverage-equivalent substitutions (same path/branch/frame-format/error-mapping) are impl-judgment.

Advisor validation protocol:

1. Read the plan-specified test definition (the one being swapped OUT)
2. List the invariants it uniquely proves: branch X, error shape Y, assertion on field Z, frame format W
3. Read the proposed replacement test
4. For each invariant in (2): does the replacement prove it? If any drops, the substitution is NOT coverage-equivalent
5. Green-light path: "all invariants preserved, substitution accepted"
6. Red-flag path: "invariants dropped: [list]. Options: (a) keep plan-specified test, (b) amend the replacement to cover dropped invariants, (c) advisor accepts coverage loss with rationale — record in retro"

### When impl proposes a plan amendment

Per plan §19 Notes "v1-AD-a precedent is the contract," divergences need explicit rationale. Advisor amendment protocol:

1. Read the specific §M the impl wants to change
2. Check v1-AD-a equivalent section if one exists (it almost always does per plan §2 Source)
3. Check PRD §N for the gold-standard constraint this plan §M is implementing
4. If amendment preserves PRD constraint: write plan patch on `docs/plan-amend-v1-JM-a-<slug>` branch off trunk, PR into trunk with `docs(plan): v1-JM-a — <amendment>` subject
5. If amendment contradicts PRD constraint: this is a hard ADR gate — STOP and surface to user, never silently edit

## JM-a plan structure — index for advisor navigation

| Section | Purpose | Relevant at |
|---|---|---|
| §1–2 | Summary + source citations | First read |
| §3 | Problem statement (why JM-a can't be dark-launched) | Defending against scope-reduction proposals |
| §4 | Solution statement + 6 lock-in decisions | Validating deviations |
| §7 | DQ #42/#43/#44/#46 guardrails (already in command templates) | Impl's Task 0 audit failure = escalate to command-template check, not per-plan fix |
| §8 | Flow design (before/after state diagrams) | Checking impl's mental model matches plan's |
| §10 | Patterns to mirror — byte-for-byte v1-AD-a maps | Resolving "how do I do X?" questions |
| §11 | Files to change (exact paths + operations) | Scope-creep detection |
| §12 | NOT building (explicit OUT list) | Scope-creep detection |
| §13 Tasks 0–11 | Step-by-step with MIRROR references | DoD checking, DQ-answering |
| §14 | Testing strategy (what each test proves) | Test substitution validation |
| §15 | Validation commands (DoD) | Impl self-check baseline |
| §16 | Acceptance criteria | Pre-PR gate review |
| §17 | Completion checklist | Retro kickoff |
| §18 | Risks + mitigations (8 risks with likelihood/impact/mitigation) | Anticipating advisor questions |
| §19 | Notes (advisor-contract principles) | Scope/deviation adjudication |
| §20 | Confidence score (8.5/10, rationale) | Expectation-setting |

## Related memory (load-bearing, verify before acting)

Advisor should load these in advisor-session context (not just rely on auto-memory index):

- `feedback_advisor_instruction_mismatch_stop_and_ask.md` — STOP and surface when counts/hashes disagree, don't power through
- `feedback_branch_manager_pm_split.md` — BM/impl split protocol (advisor duties fold into BM session here)
- `feedback_pr_review_triage_pattern.md` — four-bucket CR triage (used by BM hat, not advisor hat)
- `feedback_preserve_active_worktree_state.md` — never checkout-switch with pending work
- `feedback_multi_impl_coordination_runlog_not_dq.md` — scope claims in runlog; DQ for blocking questions
- `project_v1_AD_closed.md` — AD precedent baseline for pattern-mirror questions

## JM-a specific risk register

See `v1-JM-a-advisor-risk-register.md` for the 10 likely decision points the advisor will face during JM-a impl, keyed to plan task numbers.

## Session-close ritual

At the end of every advisor-session work segment (e.g. before leaving for the night), write a cold-resume brief to `.claude/PRPs/reports/v1-JM-a-advisor-resume-state.md` (overwrite pattern from AD-c precedent `v1-AD-c-advisor-resume-state.md`). The brief must state:

- Current trunk HEAD + phase branch HEAD
- PR #91 state (still open? merged?)
- Open DQ entries with advisor-lean recorded but unanswered
- Any in-flight plan amendments (branch, PR, status)
- Anticipated next advisor action at resume

This pattern is what let AD-c resume cleanly after an overnight break.

## What this advisor does NOT do

- Push to `phase-v1-JM-a` (impl's lane)
- Edit files under `crates/**`, `migrations/**`, `tests/**` (BM rule hard-block — applies to advisor hat too)
- Merge any PR (BM hat + explicit user confirm required)
- Silently answer a DQ without `docs(decision-queue)` commit subject (attribution breach)
- Write `answered_by: "advisor"` on an entry that originated from the impl worktree via Task N commit (per DQ rule; use `answered_by: "user"` if user stated the answer in-channel, or let impl self-resolve)
