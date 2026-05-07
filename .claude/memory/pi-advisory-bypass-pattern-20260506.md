---
title: "[SUPERSEDED] pi advisory-bypass pattern — see lesson"
memory_type: deploy-note
tags: [pi,adr-compliance,workflow,github-actions,superseded]
importance: 1
created: 2026-05-06
updated: 2026-05-06
superseded_by: .claude/lessons/feedback_gha_pi_loop_postmortem.md
---

# Superseded — do not use this note as guidance

The original 2026-05-06 mid-loop draft of this note hardcoded a
broken bypass pattern (`pr_num="${PR_NUMBER:-${GITHUB_EVENT_NUMBER:-}}"`)
as canonical. Reading it caused subsequent pi sessions to re-make the
same mistake. The corrected facts and the canonical bypass shape now
live in:

> `.claude/lessons/feedback_gha_pi_loop_postmortem.md`

That lesson is in the Brehon `.claude/lessons/` corpus, so both pi
(via PMD search once `start-pi.sh` exports `PROJECT_MEMORY_DB`) and
Claude Code (via the memory-injection rule that loads
`.claude/lessons/` at task start) read from a single source of truth.

For pi sessions that need to edit `.github/workflows/*.yml` or
`.github/scripts/*.sh`, prefer delegating to the `ci-debug`
subagent (`.pi/agents/ci-debug.md`) rather than loading the
GitHub-Actions detail into the main session — see
`.pi/PROJECT_CONTEXT.md` §"Project subagents — delegate, don't load".

This pointer exists so a pi session that recalls the file's name
doesn't load the broken-original — it lands on this redirect instead.
