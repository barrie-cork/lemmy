# Brehon Fork — Reference

Read on demand when planning or implementing. This file is NOT auto-loaded — `CLAUDE.md` stays lean and points here for lookups.

## Orchestration model — full detail

CLAUDE.md names four roles (Advisor + Planning + Impl + BM) and points at this file for the operational detail. The canonical source-of-truth is `.claude/rules/advisor-orchestrator.md` (loads at session start with the rest of `.claude/rules/`); subagent contracts are in `.claude/agents/{planning,impl-task,bm-task}.md`.

**Model selection per role** (per `.claude/agents/<name>.md` frontmatter):

| Role | Subagent file | Model | Color | Primary purpose |
|---|---|---|---|---|
| Advisor | (not a Junior subagent — runs as the persistent CC session on laptop) | Opus 4.7 (1M) | n/a | Meta-oversight: queue Junior tasks, triage DQ, run DoD smoke tests, surface user gates. Never authors content. |
| Planning | `.claude/agents/planning.md` | `claude-opus-4-7` | purple | Author plan files. Reads PRD + ADRs + lessons; runs Explore agents; commits one plan file. |
| Impl-task | `.claude/agents/impl-task.md` | `claude-sonnet-4-6` | green | Execute one task from an approved plan. Pattern-following from MIRROR refs. Per-task validation gate. |
| BM-task | `.claude/agents/bm-task.md` | `claude-haiku-4-5` | (default) | Single `/bm-*` shape — branch op, PR, CR triage, runlog write. |

**Dispatch mechanism:** the advisor calls `mcp__junior-brehon__create_task(description: "[role:planning] <slug> — see .claude/PRPs/briefs/<file>.md")`. Junior on the EliteDesk reads the description, matches the leading `[role:X]` token against the subagent's `description` frontmatter (which mentions the same token), and dispatches to that subagent in a fresh worktree. **The `[role:X]` token is a brief-content convention, not a Claude Code feature** — Claude Code's actual subagent selection mechanism is description-field matching. See `.claude/rules/advisor-orchestrator.md` "Junior task description template."

**User-facing kickoff:** `/start-brehon` slash command in the `homeserver/` repo (CWD must be `homeserver/` for the `mcp__junior-brehon__*` MCP to load). Pulls live state from authoritative sources (git, gh, decision-queue, runlog, briefs/plans/retros, Junior daemon) and synthesizes a one-screen status report with a suggested next action. Read-only.

**User gates (mandatory, advisor never skips):** plan approval, judgment-heavy DQ entries (ADR-affecting), CR triage approval, merge confirm, retro sign-off. Per `advisor-orchestrator.md` "Mandatory user gates."

**Lessons corpus:** `.claude/lessons/` (promoted from laptop PMD). Both the advisor and Junior subagents read these — same source of truth, two read paths (raw repo for subagents, search-indexed via memory MCP for advisor authoring). Per the one-system-memory principle.

**Opt-in scope:** v1-JM-d onward. Foreground use of `/prp-core:prp-plan` / `/prp-core:prp-implement` from a hand-driven session in this CWD remains valid for one-off work. Phases 1–6 + v1-AD-* + v1-JM-{a,b,c} all shipped on the foreground model.

**Custom orchestration disclaimer:** the multi-repo "homeserver CWD orchestrates brehon-fork via SSH + Junior daemon" pattern is project-specific, not a documented Claude Code workflow. `--add-dir` does NOT load another repo's `.claude/` config — that's why `homeserver/.claude/rules/advisor-orchestrator.md` is a deliberate copy of `brehon-fork/.claude/rules/advisor-orchestrator.md`. Surface a diff at session start if they go out of sync.

## Authoritative design docs

The design docs are vendored into this fork under `docs/brehon-law-inspired-network/` (the canonical source per commit `e960a128c`). Always reference them with the fork-local relative paths below:

| Priority | Path | Purpose |
|---|---|---|
| P0 | `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` | Phase-by-phase blueprint, cross-cutting requirements, test strategy, Monday-morning checklist |
| P0 | `docs/brehon-law-inspired-network/04-data-model-and-api.md` | Tables, enums, Diesel structs, DTOs, routes |
| P0 | `docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md` | 11-endpoint v0 scope + v0 simplifications |
| P0 | `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` | 15 committed ADRs (hard constraints) + 12 open questions |
| P1 | `00-README.md`, `01-vision-and-principles.md`, `02-domain-model.md` | Vision, glossary, lifecycles |
| P1 | `03-architecture.md` | Crate layout, plane separation, flow diagrams |
| P1 | `06-security-and-threat-model.md` | §2.2.1 emergency-remove, §6.1 GDPR, §7 threat table |
| P1 | `07-operations-and-federation.md` | Docker Compose on one host for v0 |

**Do NOT read** `chat1.md`, `chat2.md`, or `.docx` files. They are archival.

## Crate layout (governance additions)

Expected governance paths per [03 §7](../docs/brehon-law-inspired-network/03-architecture.md):

- `crates/db_schema/src/source/governance/*.rs`
- `crates/db_views/governance_case/`, `crates/db_views/jury_queue/`, `crates/db_views/governance_modlog/`, `crates/db_views/reputation/`
- `crates/api/api_common/src/governance.rs`
- `crates/api/api/src/governance/*.rs`
- `crates/api/api_crud/src/governance/*.rs`
- `crates/api/routes/src/governance.rs`
- `crates/apub/objects/src/governance/*.rs`
- `crates/apub/activities/src/governance/*.rs`
- `crates/apub/apub/src/governance/*.rs`
- `crates/server/src/governance.rs`
- `migrations/{timestamp}_governance_*/up.sql` + `down.sql`
- `tests/e2e.rs` — new governance integration tests (separate from the Lemmy-native `api_tests/` which is TypeScript/Jest)

## Slash commands (`.claude/commands/prp-core/` + `.claude/commands/bm/`)

Brehon-customised PRP commands + 9 BM (branch-manager) commands. Run `/` in Claude Code to see them all. Grouped by purpose:

| Command | Purpose |
|---|---|
| `/prp-plan` | Create a Brehon-aware, Rust-appropriate implementation plan keyed to the design docs and IMPLEMENTATION-PLAN-v0.md |
| `/prp-prd` | Generate a sub-PRD for a v0 feature or deviation that needs its own design document |
| `/prp-implement` | Execute a Brehon implementation plan with rigorous cargo-based validation loops |
| `/prp-review` | Comprehensive PR review: cargo validation, ADR compliance, cross-cutting invariants |
| `/prp-debug` | Deep root cause analysis for Brehon Rust bugs — finds the actual cause, not just symptoms |
| `/prp-issue-investigate` | Investigate a GitHub issue; produce an artifact for `/prp-issue-fix` |
| `/prp-issue-fix` | Implement a fix from an investigation artifact — Rust changes, cargo validation, PR |
| `/prp-codebase-question` | Research codebase questions using parallel `Explore` agents; documents what exists |
| `/prp-review-agents` | Upstream-verbatim multi-agent review flow (not Brehon-customised; may drop later) |
| `/bm-cut` | Cut a new phase or plan branch off `governance-v0` trunk |
| `/bm-push` | Push the current phase or plan branch to origin |
| `/bm-pr` | Open a PR from current phase/plan branch into `governance-v0` |
| `/bm-poll-cr` | Poll CodeRabbit reviews; write/update findings YAML |
| `/bm-prp-review` | Run Brehon `/prp-review` and merge findings into the YAML |
| `/bm-triage` | Re-classify findings into four buckets; draft digest comment |
| `/bm-merge` | Final pre-merge gate; ASKS before `gh pr merge` |
| `/bm-ping` | Send a Telegram notification ping |
| `/bm-status` | Show current branch, unpushed commits, PR state, CR count, DQ pending |

All four Tier 1 PRP commands (`prp-plan`, `prp-implement`, `prp-prd`, `prp-review`) begin with a `<brehon-context>` block that reads the design docs above before doing anything else, and they use the built-in `Explore` subagent. If a command is missing Brehon context, it's Tier 3 and intentionally verbatim from upstream.

## Monday-morning checklist

Before starting any coding session, confirm:

1. [ ] `git fetch upstream && git log upstream/main..HEAD --oneline` — any new upstream commits since last rebase?
2. [ ] `git status` — clean working tree
3. [ ] `git branch --show-current` — on `governance-v0` or a feature branch rebased onto it
4. [ ] `cargo check --workspace` — baseline builds clean (run once after any upstream rebase)
5. [ ] Read [IMPLEMENTATION-PLAN-v0.md](../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) §6 "Monday-morning checklist" for the current phase
6. [ ] Run `/prp-plan "<phase or feature>"` to produce a Rust-appropriate plan before writing any code

**Never write Rust code without a plan file committed to `.claude/PRPs/plans/`.** Planning and implementation are separate phases.

## Research tools

For ecosystem-survey OQs (multiple competing options, evolving landscape, clear evaluation criteria), use **Perplexity deep research** rather than manual search iteration:

1. **Craft a structured prompt** — include: the decision to make, the specific use-case constraints (deployment model, language preference, scale), the specific evaluation questions (version support, known bugs, resource footprint, community status). The more specific the criteria, the more actionable the output.
2. **Run in Perplexity** — use the "Deep Research" mode. Typical turnaround: 3–5 min, 30–50 sources.
3. **Copy result to `docs/research/<slug>-<year>.md`** — tracked in git alongside the OQ resolution.
4. **Synthesise into OQ resolution** — read the report, check any "verify before committing" items (e.g. open GitHub issues) directly via `gh issue view`, then close the OQ in `99-decisions-and-open-questions.md`.

**Total cost per OQ:** ~15–20 min. First-pass actionable on well-scoped questions.

**When to use:** any OQ with a "parked; re-take at X time" note where the question is about a technology's maturity, version support, bug status, or ecosystem landscape. Not needed for design/architecture OQs that are resolved by reasoning from ADRs.

**Prior uses:**
- `docs/research/matrix-homeserver-selection-2026.md` — resolved OQ-V2-10 (Matrix homeserver choice: Tuwunel). 42 sources, confirmed two active bug fixes via `gh issue view`. 2026-06-01.

**Applicable upcoming OQs:** OQ-027 (Autonomi viability for governance-log anchoring), OQ-ADR016-01/02/03/04 (cross-app federation contract protocol shapes, when M1 is scheduled).

## Upstream-rebase discipline

Per IMPLEMENTATION-PLAN-v0.md §7.1 top risks:

- Rebase onto `upstream/main` weekly. Do it on a throwaway branch first, run `cargo check --workspace` + `cargo test --test e2e` before merging the rebase back into `governance-v0`.
- Pin every weekly rebase commit in `AGPL-NOTICE.md` so we can diff governance-touching changes upstream.
- If upstream changes a Diesel schema assumption, expect governance migrations to need rewriting — treat it as a `/prp-debug` trigger.

## PR review discipline

CodeRabbit Critical (🔴) findings on Brehon PRs are **block-merge, not advisory**. Fix in the same PR, with a regression test in the same commit. Major (🟠) findings are negotiable carry-forward — file a GH issue and link from the retro. Pattern established 3× (PR #4 sanction-scoping, PR #46 #15 dup-federation, PR #46 #19 actor-binding); local advisor + e2e have demonstrated they cannot catch this class of bug.

## What NOT to build in v0

Explicitly out of scope (deferred to v1/v2/v3):

- Keycloak / external IdP, OpenFGA, Vault, Kubernetes, external log signer, blockchain anchoring
- Frontend UI — v0 is backend + API only (no Lemmy-UI changes)
- Rust-free planning (no "we'll figure it out in TypeScript later")
- Any endpoint not listed in [05 §2](../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md)

If a scope-creep idea appears, write it as an open question in a plan file — don't silently implement it.
