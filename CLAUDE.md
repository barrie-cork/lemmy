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

## Where to look next

- **Planning / design docs / crate layout / slash commands / Monday-morning checklist / out-of-scope list:** read `.claude/brehon-reference.md` on demand.
- **Design doc hub:** `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` (§3 phase-by-phase) and `04-data-model-and-api.md` (tables, DTOs, routes).
- **ADRs + open questions:** `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`.

**Never write Rust code without a plan file in `.claude/PRPs/plans/`.** Planning and implementation are separate phases.
