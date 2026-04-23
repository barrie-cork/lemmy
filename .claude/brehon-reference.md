# Brehon Fork — Reference

Read on demand when planning or implementing. This file is NOT auto-loaded — `CLAUDE.md` stays lean and points here for lookups.

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
| `/prp-pr` | Create a PR targeting `governance-v0` by default |
| `/prp-commit` | Stage + commit with a clean message (Rust-aware file globs) |
| `/prp-review-agents` | Upstream-verbatim multi-agent review flow (not Brehon-customised; may drop later) |
| `/prp-ralph` | Upstream Ralph loop (verbatim; may drop later) |
| `/prp-ralph-cancel` | Cancel an active Ralph loop (verbatim) |
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
