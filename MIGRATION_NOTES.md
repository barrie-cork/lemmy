# Pi-Coding Migration Notes — Lemmy/Brehon

Date stood up: 2026-05-04
End state: dual support (Claude Code + pi-coding side-by-side)
Working tree approach: in-place branch `trial/pi-coding`
Repo type: code-repo/application
Provider/model: `openai-codex` / `gpt-5.5`

## Stages completed

- ✓ Stage 0 — Pre-flight (`pi --version` 0.72.1; OpenAI Codex models available)
- ✓ Stage 1 — In-place branch created: `trial/pi-coding`
- ✓ Stage 2 — No destructive Claude pruning; dual-harness support preserved
- ✓ Stage 3 — Existing `.claude/skills` validated cleanly (0 issues)
- ✓ Stage 4 — `.pi/` scaffolding added
- ✓ Stage 5 — Code-repo hook extension added; selected Claude hooks ported through `.pi/hooks/` + `.pi/extensions/lemmy-hooks.ts`
- ✓ Stage 6 — MCP replacement skipped; no `.mcp.json` found
- ✓ Stage 7 — Sub-agents exposed as Pi skills in `.pi/skills/`; `.claude/agents` remains intact for Claude/Junior fallback
- ✓ Stage 7b — Claude slash commands exposed as Pi prompt templates in `.pi/prompts/`
- ✓ Stage 8 — Automated validation passed
- ✓ Stage 9 — This notes file written
- ✓ Stage 10 — Best-practices audit run; one low-severity UNDERUSE follow-up only

## Dual-harness separation

Stable pi files are committed:

- `.pi/settings.json`
- `.pi/PROJECT_CONTEXT.md`
- `.pi/extensions/lemmy-hooks.ts`
- `.pi/extensions/tsconfig.json`
- `.pi/hooks/` copied hook shims invoked by the extension
- `.pi/prompts/` flattened Claude slash-command prompt templates
- `.pi/skills/` Pi skill wrappers for Claude/Junior role agents
- `start-pi.sh`

Runtime/per-machine pi state is ignored in `.gitignore`:

- `.pi/sessions/`
- `.pi/tmp/`
- `.pi/logs/`
- `.pi/*.log`
- `.pi/settings.local.json`
- generated extension JS/map/d.ts files

Claude runtime/per-machine state remains ignored by existing rules. Claude orchestration docs and skills remain committed for dual support.

## Validation log

```text
python3 -c "import json; json.load(open('.pi/settings.json'))"  # passed
(cd .pi/extensions && tsc --noEmit)                              # passed
skill validator over .claude/skills                              # 0 issues
skill validator over .pi/skills                                  # 0 issues
pi best-practices audit                                          # 1 low UNDERUSE only
```

Audit follow-up:

- Low UNDERUSE: `.pi/extensions/lemmy-hooks.ts` registers only event handlers and no manual commands/tools/providers. Deferred; current code-repo migration does not need a manual pi command yet.

## Smoke-test runbook

Run from repo root:

```bash
cd /Users/barrie/Developer/lemmy
./start-pi.sh
```

Then run the lightweight code-repo subset:

1. T1 extension binds — startup should show pi, skills, prompts, and `lemmy-hooks.ts` extension. The extension may notify that it found `.claude/rules` files.
2. T6 bash firewall — ask pi to run `rm -rf /tmp/nonexistent_pi_trial_xyz_should_be_blocked`; choose block if prompted.
3. T7 auto-commit — ask pi to make a small edit to this file and show `git log --oneline -3`; expect an `auto(pi): update MIGRATION_NOTES.md` commit.
4. T8 skill command — run `/skill:cargo-validate` or `/skill:planning`; expect no `Unknown command` error.
5. T9 prompt command — type `/bm-status` or `/prp-plan`; expect Pi autocomplete/expansion from `.pi/prompts/`.

Smoke results:

- T1: passed — pi launched on `trial/pi-coding` and loaded `lemmy-hooks.ts`.
- T6: passed — bash firewall prompted for `git clean -f --dry-run`; accepted run was dry-run only.
- T7: passed — pi edit created commit `c215cfa59 auto(pi): update MIGRATION_NOTES.md`.
- T8: passed — `/skill:cargo-validate` was recognised and started; no `Unknown command` error.

T7 auto-commit smoke passed — 2026-05-04

## 2026-05-04 alignment update

This branch now aligns with the newer migration approach used in `phd-vault-pi`:

- `.pi/settings.json` loads legacy project skills (`../.claude/skills`); Pi auto-discovers Pi-native wrappers in `.pi/skills/`.
- 12 Claude/Junior role agents are available as `/skill:<agent-name>` commands under `.pi/skills/` while preserving `.claude/agents/` for Claude Code.
- 27 Claude slash commands are available as Pi prompt templates under `.pi/prompts/`.
- Selected hooks are copied to `.pi/hooks/` and invoked by `lemmy-hooks.ts`:
  - `pre-phase-audit.sh` → session-start reminder injected into system prompt.
  - `worktree-guard.sh` → tool-call blocker for Junior worktree escapes.
  - `observation-capture.sh` → tool-result shadow telemetry.
  - `retro-check.sh` → warn-only shutdown reminder.
- `inject-dq-state.sh` logic is implemented directly in TypeScript via `before_agent_start` coordination-state injection.

Validation rerun:

```text
python3 -c "import json; json.load(open('.pi/settings.json'))"  # passed
(cd .pi/extensions && tsc --noEmit)                              # passed
python3 .../validate_skills.py .claude/skills                    # passed
python3 .../validate_skills.py .pi/skills                        # passed
```

If this Pi session was already open before the alignment update, run `/reload` or restart via `./start-pi.sh` so the new skills, prompts, and extension handlers are discovered.
