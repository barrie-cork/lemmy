# CLAUDE.md — Brehon Fork

**Fork of:** [LemmyNet/lemmy](https://github.com/LemmyNet/lemmy) @ `d1975776a` (Lemmy 1.0-beta; last rebase 2026-04-18)
**Working branch:** `governance-v0` (v0 feature work; `main` is reserved for upstream-sync rebases)
**Rust toolchain:** `1.95` · **License:** AGPL-3.0 (see `AGPL-NOTICE.md`)

A governance-enabled fork of Lemmy 1.0-beta. v0 goal: 11 new API endpoints for a Brehon-style reputation + jury workflow, tamper-evident governance log, and outbound federation of governance signals — while staying compatible with vanilla-Lemmy content federation.

## Hard constraints (do NOT re-litigate)

From `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`, the 15 ADRs are append-only. Contradicting them requires a new ADR, not a quiet edit.

- **Lemmy 1.0-beta fork**; Extism plugin host for governance hooks (ADR-012)
- **AGPLv3** inherited (ADR-011) — every release honours the source-disclosure notice
- **v0 scope = exactly the 11 endpoints** in [05 §2](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md); nothing else
- **v0 simplifications** (mandatory): 5-juror panels, quorum 3, simple majority, outbound-only federation, local hash chain, reputation-decay stub
- **Solo-dev stack:** NO Keycloak, NO OpenFGA, NO Vault, NO Kubernetes, NO external log signer, NO blockchain anchoring
- **Auth:** Lemmy's existing JWT; optional passkey MFA via `webauthn-rs`
- **Authz:** hardcoded capability checks in Rust reading `reputation_snapshot` flags
- **Governance log:** `sha2` hash chain via Postgres triggers; `rs_merkle` + `ed25519-dalek` for signing; key in `.env` for v0
- **GDPR from day 1:** pseudonymised `actor_pseudonym` table mandatory (ADR-015)
- **Illegal content from day 1:** `CaseStatus::EmergencyRemove` mandatory (ADR-013)
- **Federation:** content-level with vanilla Lemmy works; governance signals are fork-only AP types (ADR-014)

If a plan contradicts any of these, STOP and surface to the user. Do not silently fix in the plan body.

## Orchestration model (v1-JM-d onward)

Four roles, four files: **Advisor** (persistent CC session on laptop, runs from `homeserver/` CWD; never authors content) + **Planning** + **Impl** + **BM** (all Junior subagents on the EliteDesk daemon, dispatched via `mcp__junior-brehon__*`). Full detail in `.claude/rules/advisor-orchestrator.md` (loads at session start with the rest of `.claude/rules/`); subagent contracts in `.claude/agents/{planning,impl-task,bm-task}.md`; user-facing kickoff via `/start-brehon` in the `homeserver/` repo. Lessons corpus that subagents read lives at `.claude/lessons/` (promoted from laptop PMD per the one-system-memory principle).

This is project-specific orchestration on top of Claude Code, not a documented Claude Code workflow. Three things to know: (1) the `[role:planning]` token in Junior task descriptions is a convention the **brief content** uses — Claude Code itself selects subagents by their `description` frontmatter, so the `description` field in each agent file mirrors the `[role:X]` token; (2) the multi-repo "homeserver CWD orchestrates brehon-fork via SSH + Junior daemon" pattern is custom — `--add-dir` does not load the other repo's `.claude/` config, hence the deliberate copy of `advisor-orchestrator.md` into `homeserver/.claude/rules/`; (3) DQ visibility across worktrees needs the mid-task commit-and-push discipline in `.claude/rules/decision-queue.md` because Junior's per-task isolation otherwise traps DQ writes until finalize.

This model is opt-in for v1 sub-phases — direct foreground use of `/prp-core:prp-plan` / `/prp-core:prp-implement` from a hand-driven CC session in this CWD remains valid for one-off work, hotfixes, and any sub-phase where Junior overhead isn't worth it. Phases 1–6 + v1-AD-* + v1-JM-{a,b,c} all shipped on the foreground model; v1-JM-d is the first to use the orchestrated model.

## Where to look next

- **Planning / design docs / crate layout / slash commands / Monday-morning checklist / out-of-scope list:** read `.claude/brehon-reference.md` on demand.
- **Design doc hub:** `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` (§3 phase-by-phase) and `04-data-model-and-api.md` (tables, DTOs, routes).
- **ADRs + open questions:** `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`.

**Never write Rust code without a plan file in `.claude/PRPs/plans/`.** Planning and implementation are separate phases.
