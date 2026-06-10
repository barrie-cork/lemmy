---
name: Pi advisor role dispatches to Junior; does not author plan/code/PR directly
description: On the Brehon fork, pi sessions are advisor by default. The four-role model (advisor / planning / impl / bm) is the working pattern, not a "Claude Code only" concern. Pi does NOT author plan files, Rust code, or PRs directly — it authors briefs, queues Junior tasks, polls, gates.
type: feedback
originSessionId: m2-late-2-planning-violation-2026-06-10
---

**Rule:** on the Brehon fork, a pi session is the **advisor** role in the four-role model (`advisor / planning / impl / bm`). The advisor's job is meta-oversight: read context, author briefs, queue tasks on the EliteDesk Junior daemon, poll, run user gates, write lessons/retros/ADRs. The advisor does **NOT** author plan files, Rust code, migrations, schema.rs, or PR bodies. When the user says "begin planning X" or "implement Y", the advisor's first action is to write the brief at `.claude/PRPs/briefs/<phase>-<role>-<n>.md`, then queue the matching `[role:planning]` / `[role:impl-task]` task. The plan or code is produced by the Junior worker, not by the advisor.

**Why this matters:**

- **Failure mode:** advisor-authoring a plan from the laptop session is "fast" — the model has the full context, can write a competent 50KB plan, and the user sees a deliverable immediately. But it skips the four-role model's load-bearing properties:
  1. **Specialisation by model tier:** `planning` runs at opus-4-8 on EliteDesk; `impl` at sonnet-4-6; `bm` at haiku-4-5. Each tier is matched to the task's reasoning load. The advisor at opus-4-7 on the laptop is not the right tier for either planning (which has its own opus-4-8 budget) or impl (which should be cheap per-task context).
  2. **Workspace isolation:** impl runs in a lane-dedicated worktree (`C:/Users/barri/Developer/brehon-fork-<lane>`) on the EliteDesk daemon, not in the canonical `brehon-fork` checkout. The advisor writing code from the laptop CWD violates the lane-worktree discipline and risks contaminating `governance-v0` with unmerged work.
  3. **Brief as audit trail:** the brief at `.claude/PRPs/briefs/<phase>-<role>-<n>.md` is the canonical record of what the advisor asked for. When the advisor authors the deliverable instead, there is no brief, no scope statement, no required-reading list, no constraints block — the gate between "user asked X" and "code that does X" disappears.
  4. **User gates:** the four-role model has 6 mandatory user gates (plan approval, judgment-heavy DQ, CR triage, merge confirm, Phase 2 e2e, retro sign-off). Advisor-authoring bypasses gate 1 (plan approval) because there is no plan-from-Junior to approve; it implicitly trusts the advisor's own write instead of a user-approved plan.
- **Confirmed:** m2-late-2 planning session, 2026-06-10. User said "begin planning m2-late-2". The correct response was: write the brief (done — `m2-late-2-planning-1.md` ✓), run `/brehon-clarify`, queue `[role:planning]` on Junior, poll, gate. The actual response: wrote the brief, then authored the entire plan file (`.claude/PRPs/plans/m2-late-2.plan.md`, 51KB) directly. Skipped `/brehon-clarify`. Skipped §2.4 / §2.4a / §3.1.1 / §3.3 / §3.5a gates. Wrote the plan to a working branch with a dirty `.pi/` tree from a prior session that should have been the first thing inspected. Did not surface plan approval as a user gate.
- **Cost:** the plan file is a reasonable document but it is in the wrong place (advisor CWD on a worktree, not on `governance-v0` per the four-role model) and at the wrong tier (opus-4-7 on laptop instead of opus-4-8 on EliteDesk). The user had to call out the violation explicitly. If this becomes the default, the four-role model is hollowed out.
- **Pattern across both harnesses:** this is a *role-discipline* rule, not a *harness* rule. The same rule binds Claude Code advisor sessions: the advisor on the laptop queues Junior, never authors content. The fact that pi and Claude Code both expose a "you can write files" tool surface makes the rule easy to violate on both.

**How to apply:**

- **When the user says "begin planning X" / "plan X" / "draft a plan for X":**
  1. Author the brief at `.claude/PRPs/briefs/<phase>-planning-1.md` (per CLAUDE.md §2.1 / advisor-orchestrator.md §2.1).
  2. Run `/brehon-clarify <brief-path>` if scope is non-trivial (default).
  3. Queue `[role:planning] <slug> — see .claude/PRPs/briefs/<file>.md` on Junior. Dispatch string under 100 chars.
  4. Poll the task. When it lands: read the plan, run §3.4 DoD smoke test (every §15 command literally) + §3.5 watchpoint specificity gate, surface to user (gate 1: plan approval).
  5. After approval: queue `bm-cut` to create the phase branch. NEVER write the plan yourself.
- **When the user says "implement Y" / "make change Y" / "fix Z":**
  1. Verify the plan file exists at `.claude/PRPs/plans/<phase>.plan.md`. If not, route back to the planning step above.
  2. Author the impl-task brief (per §2.2 — four sections: dispatch line, scope, required reading, constraints). Apply §2.4 file-class lesson injection table. Apply §2.4a ADR-constraint load-bearing clause if any governance file is in the file list.
  3. Queue `[role:impl-task]` on Junior. Brief must be visible on the phase branch the worker forks from (Mode A: author on the phase branch; Mode B: author on trunk + SSH-merge).
  4. Poll. Run §3.9 verify gate. Surface to user.
- **Allowed advisor-authored writes (the exceptions):**
  - Briefs at `.claude/PRPs/briefs/<phase>-<role>-<n>.md` (advisor's own work product)
  - Lessons at `.claude/lessons/{feedback,reference}_*.md` (cross-harness corpus)
  - Retros at `.claude/PRPs/reports/<phase>-retro.md` (post-impl meta-work)
  - Verify reports at `.claude/PRPs/reports/<phase>-verify.md` (post-impl meta-work)
  - ADRs at `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` (append-only; gate 2 / judgment-heavy DQ applies)
  - Open-question updates at the same ADR file
  - Handover files at `.claude/PRPs/handovers/<phase>-<scope>-<date>.md` (pre-compact discipline)
  - DQ entries at `.claude/decision-queue.json` (advisor's own orchestrator surface)
  - Project-context / AGENTS.md updates that reflect harness config (this lesson's home)
- **Disallowed advisor-authored writes (the rule):**
  - Plan files at `.claude/PRPs/plans/<phase>.plan.md`
  - Any Rust file at `crates/**/src/**/*.rs`
  - Any migration at `migrations/**/*.sql`
  - Any `crates/db_schema_file/src/schema.rs` regen output
  - PR bodies, branch cuts, merge operations (use `[role:bm-task]`)
  - `git commit` of source-code changes authored in the advisor CWD
- **Pre-compact / session-end rule:** if the advisor authored a deliverable (brief, lesson, retro) but the next session is not yet underway, the deliverable MUST be committed + pushed to the branch the worker will fork from. Briefs in working tree only = invisible to the next-session worker. (Per CLAUDE.md "Pre-queue git pre-flight" + advisor-orchestrator.md "Pre-compact handover discipline".)
- **Harness-side enforcement:** `AGENTS.md` is the canonical pi entry point and should reflect this rule. `.pi/PROJECT_CONTEXT.md` mirrors the four-role model from the pi advisor's perspective so the pi harness has the relevant context without loading `CLAUDE.md`. This lesson is the cross-harness artifact; AGENTS.md and PROJECT_CONTEXT.md are the harness-specific surfaces.

**Retirement condition:** ten consecutive advisor sessions on this repo with no advisor-authored plan/code/PR writes, verified by `git log` showing every `crates/**` and `migrations/**` change authored via Junior commit (committer email matches the Junior daemon identity), with no PR opened directly from the advisor CWD. Until then, this rule is mandatory for every Brehon pi session.
