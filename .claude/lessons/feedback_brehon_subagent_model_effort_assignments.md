# Brehon subagent model + effort assignments

## Canonical assignments (current as of 2026-05-31)

| Role | Model | Model ID | Effort |
|---|---|---|---|
| BM (branch-manager) | Claude Haiku 4.5 | `claude-haiku-4-5-20251001` | low |
| impl-task | Claude Sonnet 4.6 | `claude-sonnet-4-6` | medium |
| planning | Claude Opus 4.8 | `claude-opus-4-8` | xhigh |
| ci-watcher | Claude Haiku 4.5 | `claude-haiku-4-5-20251001` | (default) |

## Why

Opus 4.8 for planning: plan-shaping is the heaviest reasoning role; cost justified by upstream work it eliminates.
Sonnet 4.6 for impl: pattern-following from MIRROR refs, not heavy reasoning; Sonnet is the right cost/quality trade-off.
Haiku 4.5 for BM + ci-watcher: mechanical git/gh ops and polling; no heavy reasoning required.

## How to apply

When authoring a brief, confirm the dispatch string uses the role's assigned model ID. The restore script (`scripts/restore-junior-server-patches.sh`) re-patches the daemon's `.env` with `ANTHROPIC_MODEL` per role when upstream pull resets it. Check `PATCH-MARKER.md` for drift after any upstream rebase.

## See also

- `CLAUDE.md` §"Four-role model" — role definitions
- `homeserver/scripts/restore-junior-server-patches.sh` — model-env restore script
- `feedback_brehon_anthropic_only.md` — Anthropic-only constraint (non-Anthropic in logs = drift)
