# PRP Upstream Pin

This repo's PRP workflow (`.claude/commands/prp-core/`, `.claude/agents/`, `.claude/skills/prp-ralph-loop/`, `.claude/hooks/`, `.claude/PRPs/scripts/`) is forked from [Wirasm/PRPs-agentic-eng](https://github.com/Wirasm/PRPs-agentic-eng).

## Current pin

- **Repo**: `Wirasm/PRPs-agentic-eng`
- **Branch**: `development` (upstream default)
- **Commit SHA**: `9581e15df50065c591bcad1215841af2162ec74b`
- **Installed**: 2026-04-14

## What was installed from this SHA

| Upstream path | Local path | Customised? |
|---|---|---|
| `.claude/agents/code-reviewer.md` | `.claude/agents/code-reviewer.md` | No (verbatim) |
| `.claude/agents/code-simplifier.md` | `.claude/agents/code-simplifier.md` | No (verbatim) |
| `.claude/agents/comment-analyzer.md` | `.claude/agents/comment-analyzer.md` | No (verbatim) |
| `.claude/agents/docs-impact-agent.md` | `.claude/agents/docs-impact-agent.md` | No (verbatim) |
| `.claude/agents/pr-test-analyzer.md` | `.claude/agents/pr-test-analyzer.md` | No (verbatim) |
| `.claude/agents/silent-failure-hunter.md` | `.claude/agents/silent-failure-hunter.md` | No (verbatim) |
| `.claude/agents/type-design-analyzer.md` | `.claude/agents/type-design-analyzer.md` | No (verbatim) |
| `plugins/prp-core/skills/prp-ralph-loop/SKILL.md` | `.claude/skills/prp-ralph-loop/SKILL.md` | Yes — Rust/cargo validation commands |
| `.claude/hooks/prp-ralph-stop.sh` | `.claude/hooks/prp-ralph-stop.sh` | No (verbatim, `chmod +x`) |
| `.claude/hooks/README.md` | `.claude/hooks/README.md` | No (verbatim) |
| `.claude/PRPs/scripts/invoke_command.py` | `.claude/PRPs/scripts/invoke_command.py` | No (verbatim) |
| `.claude/PRPs/scripts/prp_workflow.py` | `.claude/PRPs/scripts/prp_workflow.py` | No (verbatim) |

## Explicitly NOT installed (and why)

- `.claude/agents/gpui-researcher.md`, `web-researcher.md` — upstream-specific (Zed/GPUI), no Brehon command references them
- `.claude/agents/codebase-explorer.md`, `codebase-analyst.md` — all Brehon commands route codebase research through the built-in `Explore` subagent
- `.claude/skills/prp-core-runner/SKILL.md` — Claude Code's native slash command invocation already covers this
- `PRPs/templates/`, `PRPs/ai_docs/` — no command references them; Brehon uses the design docs under `C:\Users\barri\Developer\homeserver\docs\research\brehon-law-inspired-network\` instead

## Refresh procedure

To pull newer upstream changes:

```bash
NEW_SHA=$(gh api repos/Wirasm/PRPs-agentic-eng/commits/development --jq .sha)
git diff --stat \
  $(pwd)/.claude/agents/code-reviewer.md \
  <(curl -fsSL "https://raw.githubusercontent.com/Wirasm/PRPs-agentic-eng/$NEW_SHA/.claude/agents/code-reviewer.md")
# ...repeat for each file, decide whether to accept upstream changes
```

After refreshing, update the SHA and date at the top of this file.

## Customisation policy

- **Tier 1 commands** (`prp-plan`, `prp-prd`, `prp-implement`, `prp-review`): Brehon-customised, diverge from upstream freely. See [CLAUDE.md](../CLAUDE.md) for design-doc links they must read.
- **Tier 3 commands** (`prp-review-agents`) and everything listed in the table above: keep as close to upstream as possible. Only customise for language/tooling (Python/TS → Rust/cargo). Do not rewrite logic.
- **Drift rule**: if a Brehon command starts referencing something not listed above, either add the dependency and re-pin, or change the command to use a built-in instead.
