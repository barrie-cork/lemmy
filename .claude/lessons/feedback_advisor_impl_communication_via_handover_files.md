---
name: All advisor↔impl communication goes through handover files (MUST)
description: Cross-session messages between advisor and impl sessions are written to .claude/PRPs/handovers/<role>-<date>-<topic>.md, never relayed inline via the user. The user is the carrier (relays the filepath), not the message body.
type: feedback
originSessionId: 9690351b-28f8-4817-b247-7ccf4a42b2b2
---
**Rule:** Every cross-session message between advisor and impl (in either direction) MUST be written to a handover file at `.claude/PRPs/handovers/<role>-<date>-<topic>.md`. The user is given the filepath and relays *that*, not the message body. This is mandatory, not advisory.

**Why:** User explicitly stated 2026-04-25 after the JM-c prp-skill-triggers FF-merge handover: "This is the way you should do all communication to Imp. and visa versa Impl will do for you. This is a must." The handover file format is durable, role-attributed, file-ownership-respecting (per `.claude/rules/handover.md`), grep-able, retrospect-able, and survives session-close + cold-resume cleanly. Inline relay through the user is fragile (gets paraphrased, loses context, fragments across multiple turns), wastes the user's bandwidth (they become a transcription layer instead of a coordinator), and breaks attribution discipline (a paraphrased relay can't be cited as "advisor said X at <time>").

**How to apply:**

- **Advisor → impl**: write a brief at `.claude/PRPs/handovers/impl-<YYYY-MM-DD>-<short-topic-slug>.md`. Tell the user the filepath. Do NOT type the brief body in chat.
- **Impl → advisor**: write a brief at `.claude/PRPs/handovers/advisor-<YYYY-MM-DD>-<short-topic-slug>.md`. Tell the user the filepath. Do NOT type the brief body in chat.
- **Filename convention:** `<role>-<YYYY-MM-DD>-<topic-slug>.md`. Roles: `impl`, `advisor`, `bm` (when BM-related cross-role context is needed).
- **Length target:** 150–300 lines per `.claude/rules/handover.md`. Under 150 = probably missing required sections. Over 350 = probably pasting cargo-output / CR-dumps that belong in referenced files.
- **File-ownership boundaries:** advisor handover never writes `crates/**` / `migrations/**` / `tests/**` (per `.claude/rules/handover.md` + `branch-manager.md`). Impl handover never writes `.claude/PRPs/plans/*.plan.md` after initial commit (advisor-owned).
- **Attribution:** the brief's `**Author:**` field names the writing session. Advisor briefs preserve `answered_by: "advisor"` discipline; impl briefs NEVER self-attribute as advisor (per `.claude/rules/decision-queue.md:77-98`).
- **In-conversation summary:** after writing the brief, the only chat output is the filepath + a 1-line summary of what's in it ("filepath: X; covers: <one line>"). No body re-paste.
- **Existing skill:** `/handover-advisor` and `/handover-impl` slash commands exist (per `reference_handover_commands.md`). Use them when shape matches; otherwise write the brief directly using the `.claude/rules/handover.md` template.

**Edge cases:**

- **Quick-question relay (single sentence, no context required):** still goes through a handover file. The discipline is uniform; the floor is "is this cross-session?" not "is this big enough?"
- **DQ writes:** continue to use `.claude/decision-queue.json` per `.claude/rules/decision-queue.md`. The handover file is for *narrative* + *resume context*; DQ is for *blocking questions with structured options*. They're complementary, not redundant.
- **Telegram pings:** out-of-scope per `feedback_telegram_scope_notification_only.md` — Telegram is for event-pings only, never for message-body relay between sessions. The handover file is the only message-body channel.
- **In-the-same-session communication:** N/A — the rule is about *cross-session* communication. Within one session, you reason in chat normally.

**Detection:** if you find yourself typing more than ~3 lines in chat that are addressed to "the impl session" or "the advisor session," stop. Open a Write call to a handover file instead. The chat text was supposed to be a brief; briefs go on disk.

**Related:**
- `.claude/rules/handover.md` — schema, file-ownership boundaries, length targets, exemplar list
- `.claude/commands/handover/handover-advisor.md` + `handover-impl.md` — slash commands that automate the brief-writing
- `reference_handover_commands.md` (memory) — overview of the handover commands
- `feedback_principles_not_rules.md` — exception case: this rule IS a hard rule (binary process discipline), not a principle (judgment call). Cross-session communication has only one correct shape.
- `feedback_branch_manager_pm_split.md` — the role-split this rule operationalises
- `.claude/rules/decision-queue.md §attribution-integrity` — author-label discipline that the handover file's `**Author:**` field reflects
