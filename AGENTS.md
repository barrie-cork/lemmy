# AGENTS.md — Pi entry point for Lemmy/Brehon

This is a **dual-harness repo**:

- **Pi sessions** (you, if you're reading this) load this file plus `.pi/PROJECT_CONTEXT.md` (auto-injected by `.pi/extensions/lemmy-hooks.ts`).
- **Claude Code sessions** load `CLAUDE.md` at this same root, which contains advisor/Junior/BM orchestration that does NOT apply to pi.

## Rules for pi sessions

1. **Do not read `CLAUDE.md`** unless the user explicitly asks. It describes a four-role Junior orchestration model (advisor, planning, impl, BM) that pi sessions do not run. Reading it wastes context and risks pi emulating workflows it shouldn't.
2. **Do not read `.claude/rules/*.md`, `.claude/PRPs/`, `.claude/decision-queue.json`, or `.claude/runlog/`** unless a task clearly touches that topic. Treat `.claude/` as another team's working area.
3. **Do read** the canonical project docs when relevant:
   - `.pi/PROJECT_CONTEXT.md` — pi-specific rules, cargo wrappers, conventions
   - `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md`
   - `docs/brehon-law-inspired-network/04-data-model-and-api.md`
   - `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` (ADRs — append-only)
4. **Never write Rust without a plan file** in `.claude/PRPs/plans/` — this is a Brehon hard constraint that applies to BOTH harnesses.
5. **Do not modify `.claude/`** as part of pi work unless the user explicitly asks. The Claude Code team owns it.

The substantive pi context (Brehon constraints, Rust commands, error conventions) lives in `.pi/PROJECT_CONTEXT.md` and is injected automatically.
