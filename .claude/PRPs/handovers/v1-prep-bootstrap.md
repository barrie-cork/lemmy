# Bootstrap prompt for next Claude Code session — v1 prep continuation

Copy-paste this into a fresh Claude Code session (in `C:\Users\barri\Developer\brehon-fork`) to resume.

---

I'm resuming v1 prep work for the Brehon governance fork. The previous session produced design artifacts but did not implement anything.

**First steps:**

1. Read the handover at `.claude/PRPs/handovers/v1-prep-2026-04-19.md` — full context on what got produced and recommended next steps.
2. Read `.claude/PRPs/v1-issue-triage.md` — the bucketing of 19 open v1 GitHub issues.
3. Check current state: `git status`, `git branch --show-current`, `git worktree list`. Phase 6 should still be in `phase-6` branch in the advisor worktree.
4. Verify PRD files exist: `ls .claude/PRPs/prds/v1-*.prd.md` (should be 5 files).

**Then surface to me:**
- Any drift since 2026-04-19 (new commits on `governance-v0`, Phase 6 status changes, new GH issues, merged PRs).
- Confirm Wave 1 plan is still valid (4 parallel batches: CR-MECH, CR-PROBES, CR-DB-INDEX, FILTER-DTO).
- Wait for my go/no-go before spawning any implementation agents.

**Hard constraints to remember:**
- `cargo` only via `scripts/brehon/cargo-*.bat` wrappers; never pipe through `tail` (masks exit code).
- `gh pr create` always with `--repo barrie-cork/lemmy --base governance-v0` for v1 batches.
- Wave 1 batches branch from `governance-v0` directly (they're CR follow-ups, not phases).
- CR-DB-INDEX migration timestamp must be ≥ `2026-04-21-…` (Phase 6 owns the 2026-04-20 slot).
- Don't touch the Phase 6 worktree at `C:\Users\barri\Developer\brehon-fork-advisor-phase6`.

Use Opus 4.7 max-effort for orchestration; sonnet max-effort for implementation agents.
