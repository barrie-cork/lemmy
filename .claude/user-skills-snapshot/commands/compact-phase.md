When compacting this session, remember that the resumed session will AUTO-RECEIVE, in full, at session start: CLAUDE.md, every `.claude/rules/*.md` file, and the MEMORY.md index. Do NOT restate anything they already contain — the four-role model, governance/branch/PR conventions, PMD/DQ/backfill mechanics, security constraints, canonical paths, ADRs. Assume all of that is already in context. Spending summary budget on it is pure duplication.

Your summary should preserve ONLY what the auto-load CANNOT reconstruct, in this order:

1. **The most recent active thread.** Whatever this session was actually driving — a phase/lane, or harness/infra work, or an investigation; do not assume it was lane work. Capture: the current branch, the last few commits that landed (SHA + one-line each), the stage it's at, and any in-flight state (running tasks, open PRs, pending decisions, session-discovered facts that exist in no rule file). Recency wins — older threads summarise lossily; the active thread survives intact.

   If the active thread is a `/auto-phase` run (a `.claude/auto-state/<phase>.json` ledger exists for the phase named in the session), treat the ledger as the authoritative active-thread source: cite `stage_digests[-1]` (stage, outcome, next_action_hypothesis) and `last_handover_path` verbatim, rather than reconstructing branch/stage/in-flight state from conversation. The ledger survives compaction by design; the conversation does not. Keep the next-action marked a hypothesis to re-verify on resume. For non-`/auto-phase` sessions, use the existing priority-1 wording unchanged.

2. **The remaining task list, verbatim.** Every still-open TaskList item (anything not `completed`), plus the exact next-action the session was about to take — kept verbatim enough to resume the precise next step without re-deriving it. Mark the next-action as a **hypothesis to re-verify on resume**, not a fact to execute: a Junior task / CI run / PR may have changed state *during* compaction, so any "poll #N" or "task X running" line is point-in-time and may already be stale. Instruct the resumed session to re-check live TaskList / DQ / `gh pr view` / `show_task` state BEFORE acting — never trust the summary's task-status as current.

3. **The user's own messages, verbatim.** A terse list of what the user actually asked, in order. Nothing else reconstructs intent this faithfully; it is the primary anti-drift anchor.

Compression rules for everything else:
- **Committed files:** cite path + one-line outcome only. The diff is in git — do NOT reproduce file content.
- **Uncommitted in-flight edits:** reproduce content only when the resume would otherwise lose work that isn't on disk yet.
- Compress aggressively: resolved sub-threads, read-only exploration, superseded approaches, raw command output, and completed-and-verified work (one-line outcome max).

**Token ceiling: keep the entire summary under ~4,000 tokens (~16,000 characters).** If you reach the ceiling before capturing all three items, cut further — drop older threads entirely, shorten task-list entries to one line each, trim user messages to keywords only. A summary that exceeds this ceiling defeats the purpose of compacting: the resumed session starts nearly as heavy as before.

After compaction, the resumed session should answer with zero extra digging: "what was I just working on, what's the next concrete action, and what did the user ask for" — WITHOUT the summary having repeated a single fact the auto-loaded rules already carry.
