---
name: PRP slash commands — what's customised and what's verbatim
description: Tier map for the 13 .claude/commands/prp-core/ files — which are Brehon-customised, which are upstream verbatim, and the built-in Explore substitution
type: reference
originSessionId: d8c30b13-ac92-4629-baff-664040b47be1
---
13 PRP commands live at `C:\Users\barri\Developer\brehon-fork\.claude\commands\prp-core\` — copied from [Wirasm/PRPs-agentic-eng](https://github.com/Wirasm/PRPs-agentic-eng) `development` branch and tiered by customisation depth.

**Tier 1 — Heavy customisation** (full Brehon+Rust rewrites, `<brehon-context>` block at top):
- `prp-plan.md`, `prp-implement.md`, `prp-prd.md`, `prp-review.md`

**Tier 2 — Targeted edits** (Rust commands, Brehon first-suspects, Explore agent):
- `prp-debug.md`, `prp-issue-fix.md`, `prp-issue-investigate.md`

**Tier 3 — Minimal / verbatim:**
- `prp-codebase-question.md` — Brehon context block + Explore agent swap (customised, not fully verbatim)
- `prp-pr.md` — governance-v0 default base branch (customised, not fully verbatim)
- `prp-commit.md` — Rust-aware file globs (light customisation)
- `prp-review-agents.md`, `prp-ralph.md`, `prp-ralph-cancel.md` — fully verbatim from upstream; may be dropped later if unused

**Built-in Explore substitution:** the upstream commands reference three `prp-core:*` agents (`codebase-explorer`, `codebase-analyst`, `web-researcher`) that live in a separate `plugins/prp-core/` directory we did NOT copy. All Tier 1, Tier 2, and Tier 3 commands that mention those agents have been rewritten to use the built-in `Explore` subagent via `Agent(subagent_type="Explore", …)` instead. The only file that still references custom review agents by name is `prp-review-agents.md` (Tier 3 verbatim) — user can delete it if it's never invoked.

**Intentionally preserved:**
- `prp-plan.md` lines 188 and 368 reference `api_tests/src/xx.ts` — this is correct because Lemmy 1.0-beta's upstream integration tests genuinely live in `api_tests/src/*.spec.ts` (TypeScript/Jest). Brehon governance tests go into a new `tests/e2e.rs` (Rust). Both are expected to coexist.
- Several "do NOT look for `package.json`/`pyproject.toml`/`src/`" warnings in Tier 1–3 files — these are intentional exclusions for the Rust-only fork and should NOT be removed.

**Why:** these commands are what Claude Code uses when the user runs `/prp-plan`, `/prp-implement`, etc. inside the fork. They replace having to re-explain the Brehon project context at the start of every session. The upstream PRP library was designed for TypeScript/Drizzle/Zod/Vitest, so anything that didn't get rewritten in Tiers 1–3 may still leak generic JS idioms.

**How to apply:** when the user asks to "fix/improve a prp command", remember the tier map — Tier 3 files are intentionally low-touch, Tier 1 are the ones that should stay saturated with Brehon context. When troubleshooting a command misbehaving, check whether it references a `prp-core:*` agent (bug — should be `Explore`) vs `Explore` (correct).
