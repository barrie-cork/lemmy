---
name: cross-session-commit-attribution-collision
description: "When two advisor sessions share a checkout's .git/ (including via git worktree), TWO race surfaces hit unstaged work: (a) index-staging — one session's git add sweeps the other's unstaged changes into its commit; (b) working-tree-mutation — a concurrent push to the same branch + local fast-forward silently reverts unstaged edits. Both classes; recurrence ≥ 2."
type: feedback
---

# Cross-session commit attribution collision (both race surfaces)

## TL;DR

When two advisor Claude Code sessions share the same on-disk checkout OR share a `.git/` via `git worktree add` (different working trees, same index + refs + reflog), two distinct race surfaces threaten unstaged work. The races have different mechanisms and different mitigations; both ship attribution-wrong commits or silently revert work, with no hook firing and no warning surfaced. Recurrence threshold (2 incidents, two distinct race shapes) met 2026-05-22 — promote together rather than splitting into two lessons.

## Race A — index-staging collision

When two advisor Claude Code sessions share the same on-disk checkout (NOT separate worktrees), the git index is a single shared resource. Session A's `git add` operates on the working tree's current state, which includes any unstaged changes Session B has made. If A then commits, B's changes ship under A's commit subject + body — attribution silently wrong, no hook fires, no warning surfaces.

**Why:** 2026-05-22 — canonical session (governance-v0) had three unstaged tracked changes (`.claude/rules/advisor-orchestrator.md` + `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` edits) and one untracked file (`.claude/hooks/session-start-multi-lane-check.sh`) staged via `git add`. Before the canonical session's `git commit` ran, another advisor session (fed-in-c lane on the same checkout — actually different worktree, but **same shared .git/**) issued its own `git add -A && git commit` for the v1-dq-schema-r1 Task 4 retro. The other session's `add` swept the canonical session's staged files into its index. The other session's `commit` produced `c858aa7ab chore(advisor): v1-dq-schema-r1 task 4 — four-role retro` whose tree contained: (a) the intended retro file + (b) my hook + (c) my advisor-orchestrator.md edit + (d) my bootstrap-checklist edit. The canonical session's subsequent `git commit` reported "nothing added to commit but untracked files present" because the staged changes had been swept into the prior commit. Net: my work shipped on origin (✓), under the wrong commit subject (✗), and my intended commit message documenting the A+C design + dogfood result was lost.

**Detection:** `git log -1 --stat` shows file changes whose subject + body don't describe them. The contradiction (subject = "retro" but file list includes a hook + a rule edit) is the signal. Future retro should look for this pattern.

**How to apply:**

1. **Two sessions sharing one `.git/` is the root vulnerability.** Per `.claude/rules/multi-lane-worktree.md`, lane-dedicated worktrees (`brehon-fork-<lane>`) share the parent checkout's `.git/` via `git worktree add`. Even though working-tree files are isolated, the index, refs, and reflog are shared. A `git add -A` in one worktree only stages that worktree's working-tree changes — BUT if the other worktree had `git add`-ed files already in the index, those remain staged until commit. The race window: between Session A's `git add` and Session A's `git commit`, Session B's `git add` enters → Session B's `git commit` ships A's staged files under B's subject.
2. **Mitigation (cheap):** `git status` immediately before `git commit`, after `git add`, to verify staged files match intent. If the staged file list contains entries unrelated to your work, abort the commit, run `git restore --staged <unrelated-file>`, re-verify, then commit.
3. **Mitigation (structural, future):** a PreToolUse hook on Bash matching `git commit` could compare the current `git diff --cached --name-only` against a session-scoped "intended files" list (written at `git add` time). If unrelated files are staged, refuse the commit and surface the breach. Cost: per-session state file (gitignored), small bash hook. **Implemented 2026-05-31:** `.claude/hooks/pre-commit-session-guard.sh` (warn-only PreToolUse hook). Wired in `.claude/settings.local.json`. Add `.claude/.staged-intent` (gitignored) listing intended files to suppress false-positive warnings.
4. **Forensic recovery:** if the collision is detected post-commit, the corrective action is a `docs(attribution): clarify <SHA> bundled two intents` follow-up commit (per pattern of `95f341fbb docs(attribution): clarify b004856df bundled two intents`). Do NOT amend or force-push to "fix" the attribution — the SHA is now referenced by other commits / origin / mirrors.

## Race B — working-tree-mutation revert (added 2026-05-22)

When a sub-agent's `Edit` lands an unstaged change in the working tree of a worktree whose `.git/` is shared with another active session, and the other session pushes a commit to the same branch in the interim, the next `git pull` / fast-forward (implicit or explicit — many workflows auto-FF on session-start, on a `git status` against an out-of-date ref, or on the next git tool call that walks refs) silently aligns the local working tree to the new remote state. **An unstaged edit that conflicts with the new tree state is reverted with zero warning** — no merge prompt, no abort, no "your local changes would be overwritten by merge" because the change was never staged. The race window: between the `Edit` tool returning and a subsequent `git add` (or the next git operation that touches refs).

**Why:** 2026-05-22 — parent advisor session dispatched three `general-purpose` sub-agents in parallel. Sub-agent A landed an unstaged edit at line 389 of `.claude/rules/advisor-orchestrator.md` (a new Falsifiable-hypothesis-gate bullet under §5.4) and returned a report saying "Edit landed at line 389, file modified but not staged." Before the parent session staged the edit, a concurrent session on a different worktree (`brehon-fork-fed-in-c`) pushed its own commit to `governance-v0`. The next git operation in the parent session fast-forwarded the local ref; sub-agent A's unstaged line-389 edit was silently reverted to match the new remote tree. The parent session's `grep -n "Falsifiable-hypothesis gate"` returned zero matches ~30s after sub-agent A's "landed" claim. Both claims were true at their respective moments; the concurrent session intervened between them.

The Race-A mitigation (`git status` between `git add` and `git commit`) does NOT catch Race B — the working-tree change is gone before staging, so the staged-file list is correctly empty and matches what was committed.

**Detection:** sub-agent's claim "edit landed at line N" returns; immediate `grep` for a distinctive string from the edit returns zero matches; `git log <branch>@{0}..` shows a recent FF advance from another author. The contradiction (sub-agent reported edit; tree contains no edit; remote advanced) is the signal.

**How to apply (Race-B mitigations):**

1. **Stage immediately after the edit lands.** When a tool call (especially a sub-agent dispatch) returns claiming an edit was made to a tracked file in a shared-`.git/` worktree, the very next tool call should be `git add <file>` — BEFORE any other git operation, before any `Bash` that might trigger a ref walk, before any other `Edit`. Staging anchors the change to the index; once staged, Race B is converted to Race A, which has a known mitigation (`git status` verify between add and commit).
2. **Verify-after-subagent-completes (belt-and-braces).** Before staging, run a `grep` for a distinctive string from the sub-agent's reported diff. If the grep returns zero matches, the working-tree edit has been reverted — re-apply the edit inline via `Edit` tool using the bullet/section text from the sub-agent's report (the sub-agent's report itself is the recovery source; do NOT re-dispatch the sub-agent). Then stage immediately. This adds one `grep` per sub-agent dispatch — negligible cost; saves catch-fire when Race B fires.
3. **Sub-agent reports describe sub-agent state at exit, NOT current parent-session state.** Internalise the invariant: when N seconds elapse between sub-agent exit and parent-session use of the reported result, the reported state may have been invalidated by any concurrent writer to the shared `.git/`. The verify-by-grep step (§How-to #2) is the discipline that closes the gap.
4. **Structural fix (future):** a PostToolUse hook on `Agent` tool returns matching sub-agent reports of file edits could auto-grep for a distinctive string before any subsequent tool call. Refuse subsequent non-`git-add` tool calls until the staging has happened. Cost: per-session state file tracking sub-agent-claimed edits, bash hook, small grep window. Not yet implemented (single occurrence on this race surface at promotion time; defer until 2nd).

## Cross-cutting

**Forensic recovery (both races):** if the collision is detected post-commit (Race A) or post-revert (Race B), the corrective action is to re-apply the edit and ship as a follow-up commit. For Race A's attribution breach, use a `docs(attribution): clarify <SHA> bundled two intents` follow-up commit (per pattern of `95f341fbb docs(attribution): clarify b004856df bundled two intents`). Do NOT amend or force-push to "fix" attribution — the SHA is now referenced by other commits / origin / mirrors. For Race B's silent revert, re-apply the edit inline from the sub-agent's report and stage immediately.

**The common root**: shared `.git/` across sessions. Multi-lane worktree topology (`brehon-fork-<lane>` sharing the canonical `brehon-fork/.git`) makes Race B reachable even when working-tree files are isolated. The canonical fix is **stage-immediately discipline** — staged changes are atomic with respect to FF resets; unstaged changes are not.

## See also

- `.claude/rules/multi-lane-worktree.md` "Hard refusals" #6 (Atomic read-mutate-commit for DQ writes from canonical checkout) — codifies the same race for `.claude/decision-queue.json`; this lesson generalises to ANY staged-or-unstaged file in a shared-`.git/` topology.
- `.claude/rules/advisor-orchestrator.md` §6.2 — verify-after-subagent-completes via grep; cites this lesson for the Race-B mechanism.
- `.claude/lessons/feedback_advisor_impl_communication_via_handover_files.md` — the broader "never share state across sessions in-band" pattern.
- `.claude/lessons/feedback_falsifiable_hypothesis_before_structural_fix.md` — adjacent "verify before trusting an inherited claim" pattern (sub-agent reports are inherited claims).
- The Race-A colliding commit: `c858aa7ab` 2026-05-22 — author "solo-dev" (no per-session distinction; commits authored by either session look identical).
- The Race-B clobber + recovery: `528cb76ee` 2026-05-22 — sub-agent A's line-389 edit reverted before stage; re-applied inline from sub-agent's report; bundled with the rest of the deliverables.
- `.claude/PRPs/reports/session-retro-2026-05-22-parallel-subagent-dispatch.md` §"What surprised us" → Advisor #1 — the Race-B incident in full retro context.
