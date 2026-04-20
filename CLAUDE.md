# CLAUDE.md — Brehon Fork

**Working title:** Brehon Fork (final name deferred per [99 OQ-012](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md))
**Upstream:** [LemmyNet/lemmy](https://github.com/LemmyNet/lemmy) @ **`d1975776a`** (Lemmy 1.0-beta line, `git describe` → `1.0.0-alpha.12-175-gd1975776a`; last rebase 2026-04-18)
**Fork GitHub repo:** [barrie-cork/lemmy](https://github.com/barrie-cork/lemmy)
**Working branch:** `governance-v0` (all v0 feature work lands here; `main` is reserved for weekly upstream-sync rebases)
**Rust toolchain:** `1.95` (pinned in `rust-toolchain.toml`)
**Plugin host:** Extism `1.20.0` + `extism-convert` `1.20.0` confirmed in `Cargo.toml` — [ADR-012](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) assumption holds.
**License:** AGPL-3.0 (inherited from Lemmy — see `LICENSE` in repo root and `AGPL-NOTICE.md`)

---

## What this fork is

A governance-enabled fork of Lemmy 1.0-beta. The v0 goal is 11 new API endpoints that give a Lemmy instance a Brehon-style reputation + jury workflow, a tamper-evident governance log, and outbound federation of governance signals, while staying compatible with vanilla-Lemmy content federation. See [IMPLEMENTATION-PLAN-v0.md](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) §3 for the phase-by-phase plan.

---

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

---

## Hard constraints (do NOT re-litigate)

From [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), the 15 ADRs are append-only. Contradicting them requires a new ADR, not a quiet edit.

- **Fork of Lemmy 1.0-beta**; use Extism plugin system where it simplifies governance hooks (ADR-012)
- **AGPLv3** inherited (ADR-011) — every release must honour the source-disclosure notice (see `AGPL-NOTICE.md`)
- **v0 scope = exactly the 11 endpoints** in [05 §2](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md); nothing else
- **v0 simplifications** in [05 §3](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) are mandatory: 5-juror panels, quorum 3, simple majority, outbound-only federation, local hash chain, reputation-decay stub
- **Solo-dev stack:** NO Keycloak, NO OpenFGA, NO Vault, NO Kubernetes, NO external log signer, NO blockchain anchoring. Those are v2/v3
- **Auth:** Lemmy's existing JWT; optional passkey MFA via `webauthn-rs`
- **Authz:** hardcoded capability checks in Rust reading `reputation_snapshot` flags
- **Governance log:** `sha2` hash chain via Postgres triggers; `rs_merkle` + `ed25519-dalek` for signing; key in `.env` for v0
- **GDPR from day 1:** pseudonymised `actor_pseudonym` table mandatory (ADR-015)
- **Illegal content from day 1:** `CaseStatus::EmergencyRemove` mandatory (ADR-013)
- **Federation:** content-level with vanilla Lemmy works; governance signals are fork-only AP types (ADR-014)

**If a plan appears to contradict any of these, STOP and surface it to the user. Move contradictions to `§8 open questions to escalate`; do not silently fix in the plan body.**

---

## Crate layout (governance additions)

Expected governance paths per [03 §7](docs/brehon-law-inspired-network/03-architecture.md):

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

---

## Slash commands

13 PRP commands in `.claude/commands/prp-core/` — run `/` to see them. Tier 1 (`prp-plan`, `prp-implement`, `prp-prd`, `prp-review`) include a `<brehon-context>` block; Tier 3 are verbatim upstream.

---

## Monday-morning checklist

Before starting any coding session, confirm:

1. [ ] `git fetch upstream && git log upstream/main..HEAD --oneline` — any new upstream commits since last rebase?
2. [ ] `git status` — clean working tree
3. [ ] `git branch --show-current` — on `governance-v0` or a feature branch rebased onto it
4. [ ] `cargo check --workspace` — baseline builds clean (run once after any upstream rebase)
5. [ ] Read [IMPLEMENTATION-PLAN-v0.md](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) §6 "Monday-morning checklist" for the current phase
6. [ ] Run `/prp-plan "<phase or feature>"` to produce a Rust-appropriate plan before writing any code

**Never write Rust code without a plan file committed to `.claude/PRPs/plans/`.** Planning and implementation are separate phases — see [IMPLEMENTATION-PLAN-v0.md §2](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md).

---

## Upstream-rebase discipline

Per [IMPLEMENTATION-PLAN-v0.md §7.1](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) top risks:

- Rebase onto `upstream/main` weekly. Do it on a throwaway branch first, run `cargo check --workspace` + `cargo test --test e2e` before merging the rebase back into `governance-v0`.
- Pin every weekly rebase commit in `AGPL-NOTICE.md` so we can diff governance-touching changes upstream.
- If upstream changes a Diesel schema assumption, expect governance migrations to need rewriting — treat it as a `/prp-debug` trigger.

---

## PR review discipline

CodeRabbit Critical (🔴) findings on Brehon PRs are **block-merge, not advisory**. Fix in the same PR, with a regression test in the same commit. Major (🟠) findings are negotiable carry-forward — file a GH issue and link from the retro. Pattern established 3× (PR #4 sanction-scoping, PR #46 #15 dup-federation, PR #46 #19 actor-binding); local advisor + e2e have demonstrated they cannot catch this class of bug. See `feedback_coderabbit_block_merge_critical.md`.

---
