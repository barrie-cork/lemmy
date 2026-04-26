---
name: Multi-impl coordination via runlog (not DQ)
description: When multiple impl sessions work the same phase branch in parallel, use an append-only runlog at .claude/PRPs/phase-<N>-runlog/01-phase-<N>-progress.md for scope claims and hand-off markers — NOT decision-queue.json
type: feedback
originSessionId: e837bbbe-283f-4941-9ab4-afb178937118
---
When two or more impl sessions are active on the same phase branch (e.g. Impl1 + Impl2 on PR #46 CodeRabbit review cycle, 2026-04-19), coordinate via the runlog file at `.claude/PRPs/phase-<N>-runlog/01-phase-<N>-progress.md`. Append scope claims, commit SHAs, and step-sequence markers there.

**Why:** Decision-queue is for blocking questions requiring advisor input (per `.claude/rules/decision-queue.md`). Scope claims + hand-off markers are not questions — they're execution-history entries that belong in the append-only runlog. Impl1 established the pattern at phase-6 commit `250dd9066` ("chore(runlog): Impl1 coordination note") before Impl2 arrived. Impl2 initially mis-routed to DQ #39 and had to revert.

**How to apply:**

1. At session start, grep for `01-phase-<N>-progress.md` + read the tail 100 lines to see active claims.
2. When starting work, append a claim block naming files you'll touch and which CodeRabbit findings / tasks you own.
3. On each commit, append the SHA + finding/task labels to your claim block.
4. When you need to hand off to another impl (e.g. "merge6 green, other impl may push now"), commit a dedicated marker commit like `chore(runlog): Impl1 merge6 green — OK to push cosmetic sweep`.
5. Never post scope claims to `decision-queue.json` — reserve DQ for genuine blocking questions.

**Signal-SHA pattern for merge6 → cosmetic-sweep → retro sequence (Phase 6 reference):**

- Step 1: Impl1 runs merge6 (check + clippy + e2e compile + e2e run)
- Step 2: Impl1 pushes a signal commit (e.g. `3a42f43c2`) with body documenting merge6 green
- Step 3: Impl2 pushes cosmetic commits ON TOP of the signal
- Step 4: Impl1 fetches, harvests Impl2 SHAs, writes retro amendment
- Step 5: Impl1 pushes retro as FINAL commit — CodeRabbit re-review fires on that push

The signal-SHA acts as a mutex: Impl2 only pushes after seeing the signal; Impl1's retro is guaranteed to include all Impl2 SHAs.

**Anti-patterns:**

- Posting a claim to `decision-queue.json` with `"question"` containing "CLAIM — working on X" — wrong channel; DQ is not an execution-history log.
- Assuming another impl is inactive because DQ has no pending entries — they may be committing directly to the branch.
- Pushing cosmetic commits without checking for an explicit green-signal from the session running merge6 — risks invalidating validation between run and retro.

**Discovery hint for fresh sessions:** if a brief mentions "another agent is active", first action is `ls .claude/PRPs/phase-*-runlog/` and `tail -100 01-phase-*-progress.md`, not `cat .claude/decision-queue.json`.

**Generalizes to v0-polish + any post-phase parallel session (2026-04-19 PM):** The same coordination rule applies whenever multiple sessions can write to the same worktree. Today's incident: a /prp-debug session entered `brehon-fork-v0-polish` to apply an issue #48 fix while an active polish session in the same worktree was running its own cargo compile and committing. The polish session committed `2ced0761d` (their independent version of the same fix) at 19:38; my edits landed at 19:45 and were swallowed by their commit when git noticed the duplicate text. We avoided clobbering only by accident.

**Pre-flight check before touching any non-primary worktree:**

1. `git -C <worktree> status` — non-empty pending diff signals an active session
2. `git -C <worktree> log -5 --oneline` — recent commits with `polish-N` / `Impl1` / `Impl2` style markers signal an active session
3. `ps -ef | grep -iE 'cargo|rustc' | grep <worktree-name>` — running cargo process in that worktree confirms it
4. Check `.claude/build-*.log` and `.claude/check-*.log` mtimes — recent (within last hour) means active

If any of those signal an active session: **do not edit files in that worktree.** Surface to the user, ask whether to coordinate or land elsewhere. Even if the fix you have in mind is correct and small, two sessions writing the same file race the artifact lock, race git stage, and produce false-positive "diff is clean" states when one commit silently absorbs the other's edits.

**Worktrees are not isolated when an active session is running.** They share the on-disk file state and the `target/` cache. The git branch isolation does not protect you from interleaving file edits.
