---
name: Telegram scope — notification carrier only, never review content
description: Phone review misses byte-verification; diff relay loses formatting; keep Telegram to control-flow events
type: feedback
originSessionId: 2fc8749f-b29e-4ec8-9b6e-47c640c19b0e
---
When wiring Telegram for the two-session PM/impl split, scope it strictly to NOTIFICATION control-flow events. Do NOT use it for review content, diff relay, or answer composition.

**Why:** Claude (another session) initially framed Telegram as a general "out-of-band advisor surface" including CR triage and CodeRabbit findings. Testing that framing against the actual v1-AD-c review cadence revealed three failure modes:

1. **Phone review misses byte-verification.** t4's NOT5 paragraph (625 bytes, plan §10.5) and t5's DbPool import-drift both needed terminal-side byte-diff and file reads. A phone-typed "looks good" would have false-greened both.
2. **Diff relay via Telegram is lossy.** Markdown codeblocks truncate on long diffs, emoji-punctuation confusion, line-number shift from copy-paste. Diffs belong in the terminal where `git diff` renders faithfully.
3. **Telegram-content → DQ auto-answer violates attribution rules.** `answered_by: "user"` requires user stated the answer in-channel to the terminal session (per `.claude/rules/decision-queue.md` §Attribution integrity). Telegram → auto-edit DQ is a process breach.

**How to apply:**

Telegram is for **event notifications**, not content:
- ✓ "Impl staged t8, 8 e2e tests, diff 650+/5-" → just a ping so you know to open advisor terminal.
- ✓ "Advisor wrote review-stop on t5, see runlog" → ping so Impl knows to open runlog.
- ✓ "cargo test --test e2e finished, exit 0 in 11m 12s" → ping after long cargo runs.
- ✓ "CR posted 3 Critical findings on PR #77" → ping to triage in terminal.
- ✗ Full diff in Telegram message body.
- ✗ Review-go response composed from phone ("looks ok commit it").
- ✗ DQ answer content routed via Telegram → auto-edit.
- ✗ Log file tails pasted to Telegram (may contain paths, SHAs, error messages that become confusing out of context).

**Review checkpoints that REQUIRE terminal** (never flip to review-go from phone):
- Byte-verification against plan sections (like t4 NOT5 check)
- Multi-file diff reads (like t5 case_open_snapshot + mod.rs + create_report.rs)
- Log-tail inspection for wrapper-level exit-code masking (cargo-test.bat batch trap)
- Import-drift detection (retro6 pattern — Impl silently corrects; advisor must grep to verify)
- Invariant checks (governance-log registry count, entry-kind literal uniqueness)

**The human relay stays in the loop intentionally.** The bot fires a ping; you read it on phone; you decide whether it's worth walking to the terminal now or in 10 minutes. The bot doesn't act. This preserves the human checkpoint that catches "Impl just committed without review" or "advisor mis-counted a byte-diff" class mistakes.

**When to escalate phone → terminal immediately:** stage events on pause-tasks (1, 2, 3, 4, 5, 8). Those gate the commit. Auto-task events (0, 6, 7 commits) can wait until you're back at the desk.

**Relates to:** `feedback_telegram_channel_use.md` (forbidden uses: access mutations, destructive git, secrets), `project_telegram_integration_deferred.md` (when to wire), `feedback_branch_manager_pm_split.md` (the two-session split this supports).
