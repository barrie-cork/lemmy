# Advisor subagent dispatch patterns (low-recurrence, externalized from advisor-orchestrator.md)

Two patterns about `Agent` tool dispatch that are sub-promotion: each has only a single confirmed occurrence at promotion time, and per `feedback_principles_not_rules.md` single-occurrence patterns don't belong in always-loaded rules. Externalized from `.claude/rules/advisor-orchestrator.md` §6.1 + §6.2 — read on demand when the corresponding pattern fits the current dispatch.

The always-applicable §6.3 ("Bounded sub-agent dispatch and report semantics") stays inline in the rule file; it's the rule's permanent §6 content.

Companion: `.claude/rules/advisor-orchestrator.md` §6 — the always-loaded body. When the second recurrence of either pattern below occurs, lift it back from refs/ into the rule.

## Parallel dispatch for N independent deliverables (status: defer-pending-2nd-recurrence)

When the advisor session has N independent deliverables to produce (retro-followups, multi-file audits, parallel lesson-authoring, parallel research probes), dispatch all N in a **single assistant message with multiple `Agent` tool blocks**. The harness parallelises them — total wall-clock is approximately `max(per-agent runtime)`, NOT `sum(per-agent runtime)`.

**Demonstrated 2026-05-22:** three `general-purpose` sub-agents (rule promotion + lesson authoring + hooks audit) dispatched in one message ran concurrently; ~7 min wall-clock vs ~12-15 min serial. First-try usability on all three; ~3× speedup.

**When to apply:**

- The deliverables are **independent** — no agent's output is required input to another's. (Sequential pipeline → still serial.)
- Each deliverable is **bounded** — a single file edit, a single audit report, a focused research probe. Open-ended "investigate X" tasks may need iteration; harder to parallelise reliably.
- The advisor has the **synthesis context** — sub-agents return their work; the parent integrates. Don't delegate the integration step.

**Dispatch shape:** one assistant message containing K `Agent` tool blocks (K typically 2-4). Each block carries its own self-contained prompt (sub-agents see no parent conversation; brief them as if they walked into the room cold per the `Agent` tool guidance). Use `general-purpose` subagent_type unless a specialised agent fits better; pass `model: "sonnet"` for routine work (cheaper, fast enough), `model: "opus"` for synthesis-heavy work.

**Promotion status:** **defer-pending-2nd-recurrence**. The pattern worked once (2026-05-22); recurrence threshold per `feedback_principles_not_rules.md` is 2 across distinct session types. Use the pattern when it fits; record evidence in session retros; promote to formal discipline after 2nd applicable session (likely: another retro-followup batch, or a multi-file audit in a sub-phase). Per `.claude/PRPs/reports/session-retro-2026-05-22-parallel-subagent-dispatch.md` §"Promotion candidates".

## Verify-after-subagent-completes (belt-and-braces; status: record-only, single occurrence)

When a sub-agent's report claims a file edit landed in a tracked file under shared `.git/` (canonical `brehon-fork` checkout OR any `brehon-fork-<lane>` worktree), the parent advisor session MUST verify the edit still exists in the working tree **before** staging or proceeding with dependent work.

**Why:** sub-agent reports describe sub-agent state at exit, NOT current parent-session state. Between sub-agent exit and parent-session use of the report, concurrent writers to the shared `.git/` can invalidate the report. Race B per `feedback_cross_session_commit_attribution_collision.md`: unstaged working-tree edits silently reverted by concurrent push + local fast-forward state alignment.

**How to apply:**

1. Sub-agent returns claiming "edit landed at line N" or similar specific change.
2. **Immediately run a `grep` for a distinctive string** from the sub-agent's reported diff. (Distinctive = unlikely to appear elsewhere in the file by accident — pick a phrase from the new bullet, a unique identifier, a specific section heading.)
3. **Zero matches** → the edit has been reverted by a concurrent writer. Re-apply inline via `Edit` tool using the bullet/section text from the sub-agent's report. **Do NOT re-dispatch the sub-agent** — the report itself is the recovery source.
4. **Match found** → stage immediately (`git add <file>`) BEFORE any other tool call. Staging converts Race B into Race A which has a known mitigation (`git status` verify between add and commit per `feedback_cross_session_commit_attribution_collision.md`).

**Promotion status:** **record-only**. Single occurrence at promotion time (2026-05-22 sub-agent A clobber + inline-recovery). The mechanism is documented here; formal promotion to a hard `MUST` defers until 2nd applicable incident. In the interim, the pattern is in the corpus and reachable by any future session.

## See also

- `.claude/rules/advisor-orchestrator.md` §6 "Subagent delegation" — the always-loaded body (the §6 preamble + always-applicable §6.3). The one-line pointer in §6 of that file refers here.
- `.claude/lessons/feedback_principles_not_rules.md` — the doctrine that motivated the externalization.
- `.claude/lessons/feedback_cross_session_commit_attribution_collision.md` — Race-A + Race-B mitigations cited from the "Verify-after-subagent-completes" pattern.
- `.claude/PRPs/reports/session-retro-2026-05-22-parallel-subagent-dispatch.md` — wall-clock evidence + status rationale for the "Parallel dispatch" pattern.
