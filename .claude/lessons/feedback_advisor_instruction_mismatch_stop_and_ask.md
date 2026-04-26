---
name: Stop and ask when advisor instructions don't match on-disk state
description: When a Barrie-advisor message gives an instruction that contradicts the current on-disk or repo state, STOP and surface the mismatch in plain English instead of guessing which interpretation was intended. Applies to task numbers, file states, commit counts, and destructive git operations
type: feedback
originSessionId: 98b0fb05-df8a-4d08-a152-27b3f01323e4
---
When an advisor instruction (from Barrie reviewing the work) names something specific — a task number, a commit hash, a file state, a commit count — and what I see on disk or in git disagrees with the instruction, STOP and ask rather than picking an interpretation and acting.

**Why**: happened twice in Phase 1, both caught in time, both would have caused serious problems if I'd acted on the wrong interpretation.

1. **Task-number slip.** Near the end of Phase 1, Barrie said "commit task 14 with the exact test file as it was when the test passed." But the test that had just passed at that moment was `can_insert_moderation_case` (task 13), not the hash-chain test (task 14). I read it as a typo (correctly, as it turned out — Barrie confirmed), but the right move was what I actually did: stop, state both interpretations explicitly, ask which one was intended, and not touch any refs until confirmation. Had I committed "task 14" against the task 13 test content with a task-14 commit message, the archaeology trail would have been permanently corrupted in a way that's hard to fix cleanly.

2. **18-vs-17 commit count.** During the final merge gate, Barrie's instruction was `git rev-list --count origin/governance-v0..feature/phase-1-schema` should return 17; it returned 18. The extra commit was `ed46ce678 plan(phase-1): fold advisor-review pass 1 feedback` — a legitimate pre-existing commit that had landed on local governance-v0 before the Ralph loop started, never got pushed, and was inherited into the feature branch at cut-time. The three options were (a) proceed with the merge of 18 commits, (b) rebase to exclude the extra, (c) cherry-pick only the 17 I owned. Options (b) and (c) would have rewritten hashes that are canonical in Phase 1 notes across two repos, which would have been a real cost for zero benefit. The right move was to stop, surface all three options with recommendations, and wait for Barrie's explicit go-ahead on option (a) before running any destructive git command.

**How to apply**:
- When an instruction names a specific count, hash, task number, file, or state and the observed reality disagrees: stop. Don't act. Don't pattern-match to "the intended interpretation."
- Report the mismatch in plain English: what the instruction says, what the actual state is, what the candidate explanations are, what I recommend, what I need confirmed before acting.
- Offer a clear set of options with their trade-offs. Name destructive options (amend, rebase, reset --hard, force-push, cherry-pick away from the chain) explicitly so the advisor can see the cost of each.
- Wait for explicit approval before running destructive git operations — amend, reset, rebase, force-push, cherry-pick-that-rewrites-history, branch -D, push --force. These bypass git's safety rails and are not reversible via normal workflow.
- The cost of stopping to ask is low (one message round-trip). The cost of acting on the wrong interpretation is potentially unrecoverable — lost work, dead hash references in memory, corrupted phase trails across multiple repos.
- This is not timidity — it's the single most important habit the Phase 1 run demonstrated per Barrie's closing note. Carry it forward into Phase 2+ as a reflex, not an occasional check.

**Related but distinct habit**: before recommending something from memory, verify it's still accurate (the memory system's standard "trust but verify" rule). The advisor-mismatch rule is about the *current turn's instruction*, not about stale stored knowledge. Both matter. Don't confuse them.
