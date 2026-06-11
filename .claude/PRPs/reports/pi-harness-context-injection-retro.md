# Retro — pi-harness-context-injection (progressive disclosure by Brehon role)

**Date:** 2026-06-10  
**Plan:** `.claude/PRPs/plans/pi-harness-context-injection.plan.md`  
**Executor:** pi advisor session (harness-maintenance scope)  

---

## What changed

| File | Change | Before | After |
|------|--------|--------|-------|
| `.pi/extensions/lemmy-hooks.ts` | Added `MODE_CONTEXT` map, `CONTEXT_HEADINGS`, `sliceProjectContext()`, `authorizedPathsNotice()`, `filteredRuleIndex()`, `autoInjectSkill()`; rewrote `before_agent_start` | Monolithic context dump (full PROJECT_CONTEXT.md + all 25 rules + all modes same) | Progressive disclosure: per-mode slices, filtered rules, auto-injected skills, authorized paths |
| `.pi/skills/planning/SKILL.md` | Rewritten for pi-native tool set | 15.4K — Explore subagents, LSP, mcp__ref-context, Junior daemon dispatch, model enforcement | ~8.5K — rg/find/read/bash, "When loaded" preamble, pi-native exploration |
| `.pi/skills/impl-task/SKILL.md` | Rewritten for pi-native tool set | 21.6K — forbidden-window, Shape-G ci-watcher, validate-pending-laptop, EliteDesk, LSP, mcp__ref-context | ~7.3K — local cargo via wrappers, pi-native exploration, DQ writes via edit |
| `.pi/skills/bm-task/SKILL.md` | Rewritten for pi-native interaction | 9.5K — Junior daemon dispatch, EliteDesk, AskUserQuestion, model enforcement, Agent() prohibition | ~5.6K — interactive confirmation, subagent delegation, "When loaded" preamble |
| `.pi/skills/ci-watcher/SKILL.md` | Rewritten for pi-native polling | 15.3K — Junior subagent framing, model enforcement, Agent() prohibition, daemon dispatch | ~4.2K — standalone bash polling, mechanical report |
| `.pi/skills/branch-manager/SKILL.md` | Scrubbed Claude refs | AskUserQuestion, Agent(), Claude Code project-rules inheritance, "isolated context window" | pi-native confirmation, subagent delegation |
| `.pi/skills/pmd/SKILL.md` | Scrubbed MCP ref | "HTTP MCP service" | "HTTP API service" |
| 7 light skills (`code-reviewer`, `code-simplifier`, `comment-analyzer`, `docs-impact-agent`, `pr-test-analyzer`, `silent-failure-hunter`, `type-design-analyzer`) | Scrub ported-from note | Claude/Junior migration note with frontmatter caveat | Simple "Pi-native skill. Ported from Claude Code" |

## Noise reduction

| Mode | Before (PROJECT_CONTEXT.md) | After | Rules before | Rules after |
|------|------------------------------|-------|-------------|-------------|
| main-safe | Full (7 sections, ~3 KB) | Full (unchanged) | 25 | 25 (unchanged) |
| planning | Full (7 sections, ~3 KB) | 3 sections (constraints, workflow, harness, ~1 KB) | 25 | 4 |
| impl-task | Full (7 sections, ~3 KB) | 4 sections (+rust quick-reference) | 25 | 5 |
| bm | Full (7 sections, ~3 KB) | 5 sections (+subagents, setup) | 25 | 6 |
| ci-debug | Full (7 sections, ~3 KB) | 5 sections (+subagents, setup) | 25 | 2 |
| review-readonly | Full (7 sections, ~3 KB) | 4 sections (+rust quick-reference) | 25 | 4 |
| harness-maintenance | Full (7 sections, ~3 KB) | 6 sections (+rls, subagents, setup) | 25 | 3 |

**Approximate context savings:** planning mode injects ~55% fewer tokens from PROJECT_CONTEXT.md and rules index. BM mode injects ~40% fewer. impl-task mode injects ~35% fewer.

## Skill auto-injection

Three skills now auto-load when entering their mode — no manual `/skill:` step needed:

| Mode | Auto-injected skill | Key instructions now in system prompt |
|------|---------------------|--------------------------------------|
| planning | `.pi/skills/planning/SKILL.md` | FILES YAML block, complexity score, pi-native exploration, hard refusals |
| impl-task | `.pi/skills/impl-task/SKILL.md` | MIRROR refs, cargo wrapper validation, DQ mid-task discipline, commit shape |
| bm | `.pi/skills/bm-task/SKILL.md` | 9 verbs, confirmation protocol, subagent delegation, hard refusals |

Skills are injected by reading the SKILL.md and stripping YAML frontmatter. If the file is missing, the handler falls back gracefully (no injection, no error).

## Regression risk

- **main-safe unchanged:** empty `projectContextHeadings[]` and `ruleFilter[]` → full existing behavior preserved. Empty `recommendedSkill` → no auto-injection. Empty `authorizedReadPaths` → no override notice.
- **Other extensions not affected:** `before_agent_start` chains `event.systemPrompt` (doesn't replace it). The additions are appended — other extensions' injections remain.
- **Skill file reads are safe:** `fs.existsSync()` check before `readFileSync()`. Errors caught, null returned.
- **No changes to `.claude/` ownership areas** beyond the retro report (advisor-authorized).
- **Factory profile stream skipped:** `pi-harness-factory` was already removed 2026-06-10. The profile reading was a no-op.

## What was intentionally skipped

- **Stream D (factory profile):** `pi-harness-factory/` removed 2026-06-10 — no profile data to feed.
- **`.pi/agents/bm-pi.md` and `.pi/agents/ci-debug.md`:** These subagent definitions reference skills that were rewritten; they remain functional since they load the rewritten skills via `read`.
- **`.claude/agents/`:** Untouched per W5 (dual-harness constraint). The original Claude agents remain as-is for Claude Code sessions.

## Canary

Next pi session using `/brehon-mode planning` should:
1. Show only the 3 core PROJECT_CONTEXT.md sections (constraints, workflow, harness) — no cargo wrapper table, no subagent install instructions
2. Show only 4 rules in the index (handover, pmd-invariants, pmd-search-strategy, session-awareness) — not all 25
3. Include the full planning skill instructions (FILES YAML block, complexity score, pi-native exploration) in the system prompt without a manual `/skill:planning` step
4. Include an "Authorized .claude/ paths" notice naming the planning-relevant paths
5. Include the planning persona line
