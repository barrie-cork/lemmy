# Brehon pi-harness-factory profiles

These profiles are a UX layer for `/factory browse`, `/factory preview`, and `/factory use`.
They are **not** the policy authority.

Hard enforcement stays in `.pi/extensions/lemmy-hooks.ts`:

- `/brehon-mode` is the canonical mode switch.
- `pi-harness-factory` active profiles with ids `brehon-*` are mapped back to `/brehon-mode` by `lemmy-hooks.ts`.
- Path policies, Rust plan-file checks, raw-cargo blocking, secret-path blocking, and CI auto-commit suppression are enforced by `lemmy-hooks.ts`.

Use `/factory use brehon-impl-task` for the profile UI; the next prompt/tool call will sync it to `/brehon-mode impl-task`.

Use `/brehon-mode harness-maintenance` (or `/factory use brehon-harness-maintenance`) only for explicit harness metadata / Recursive Learning System work such as skill frontmatter cleanup, lesson promotion, or session-retro artifacts. This mode permits `.claude/skills/`, `.claude/lessons/`, `.claude/PRPs/reports/`, `.pi/skills/`, `.pi/scripts/`, `.pi/extensions/`, and `.pi/harness-factory/` while still blocking app code and broader `.claude/` ownership areas. After skill edits, run:

```bash
python3 .pi/scripts/validate-skills.py
```

Pi RLS parity: `.pi/extensions/lemmy-hooks.ts` runs a PMD HTTP reachability guard on session start and runs lesson frontmatter + lesson→PMD sync hooks after lesson `edit`/`write` tool calls. `pi-harness-factory` profiles are tracked; `.pi/harness-factory/active.json` is runtime-local.

For punctuation-heavy skill descriptions, prefer folded YAML:

```yaml
description: >
  Short routing-oriented description with punctuation: safe in folded block style.
```
