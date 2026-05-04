# Pi Project Context — Lemmy/Brehon

This repo is an application codebase: a governance-enabled fork of Lemmy 1.0-beta. Use pi for normal code-repo tasks unless the user explicitly asks for Claude/Junior orchestration work.

## Non-negotiable Brehon constraints

From `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`, existing ADRs are append-only. Contradicting them requires a new ADR, not a quiet edit.

- Lemmy 1.0-beta fork; Extism plugin host for governance hooks.
- AGPLv3 inherited; releases must honour source-disclosure notice.
- v0 scope is exactly the 11 endpoints in `docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md` §2.
- v0 simplifications: 5-juror panels, quorum 3, simple majority, outbound-only federation, local hash chain, reputation-decay stub.
- Solo-dev stack: no Keycloak, OpenFGA, Vault, Kubernetes, external log signer, or blockchain anchoring.
- Auth uses Lemmy's existing JWT; optional passkey MFA via `webauthn-rs`.
- Authz is hardcoded capability checks in Rust reading `reputation_snapshot` flags.
- Governance log uses `sha2` hash chain via Postgres triggers; `rs_merkle` + `ed25519-dalek` for signing; key in `.env` for v0.
- GDPR from day 1: pseudonymised `actor_pseudonym` table mandatory.
- Illegal content from day 1: `CaseStatus::EmergencyRemove` mandatory.
- Federation: vanilla Lemmy content federation remains compatible; governance signals are fork-only AP types.

If a requested plan contradicts these, stop and surface it to the user.

## Coding workflow constraints

- Never write Rust code without a plan file in `.claude/PRPs/plans/`.
- Planning and implementation are separate phases.
- Prefer reading canonical design docs before changing specs or API behaviour:
  - `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md`
  - `docs/brehon-law-inspired-network/04-data-model-and-api.md`
  - `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
- For crate layout, slash-command history, and repo-specific references, read `.claude/brehon-reference.md` on demand.

## Dual-harness boundary

The `.claude/` directory is still used for Claude Code and custom Junior orchestration. Do not delete, rename, or simplify it as part of pi work unless the user explicitly asks.

For pi sessions:

- Treat Claude/Junior orchestration docs as optional reference material, not mandatory foreground workflow.
- Use `.claude/rules/*.md` only when a task clearly touches the rule's topic.
- Do not invoke or emulate Junior subagents unless explicitly requested.
