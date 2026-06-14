# Brief: m2-late-b-actor BM-POLL-CR

## 1. Role + dispatch

`[role:bm-task] m2-late-b-actor bm-poll-cr — see .claude/PRPs/briefs/m2-late-b-actor-bm-poll-cr-1.md`

## 2. Scope

Poll CodeRabbit findings on PR #197 (`phase-m2-late-b-actor → governance-v0`) on `barrie-cork/lemmy`.

**Produce:**
- Findings YAML at `.claude/PRPs/reviews/pr-197-findings.yaml` (overwrite shell — first real poll on this PR; shell was written by bm-pr with counters at 0)
- Runlog entry in `.claude/runlog/bm-runlog.md` (create if absent)
- Summary of finding counts by bucket + severity in task output

**Do NOT:**
- Merge the PR
- Post any PR comments or submit reviews
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `tests/`, `migrations/`
- Touch `docs/` or `services/` source files

## 2. Context

CodeRabbit has completed its review on PR #197. The review is fully posted BEFORE this bm-poll-cr runs — no wait required; proceed directly to YAML authorship. Ingest ALL findings (actionable comments + any nitpick/outside-diff findings CR folded into collapsible sections) into the findings YAML.

**PR scope reminder (m2-late-b-actor — B-actor portable actor-ID linkage, ADR-016 C3):**
- `migrations/2025-05-18-000001_actor_app_link/` — `up.sql` + `down.sql` for `actor_app_link` table
- `crates/db_schema/src/source/governance/actor_app_link.rs` — Diesel model + InsertForm
- `crates/db_schema/src/schema.rs` — `actor_app_link` table! macro
- `crates/db_schema/src/newtypes/mod.rs` — `ActorAppLinkId` newtype
- `crates/api/api_common/src/governance.rs` — `LinkActorRequest`, `LinkConfirmRequest`, `LinkClaimPayload` DTOs
- `crates/api/api/src/governance/actor_app_link.rs` — `link_actor`, `link_confirm`, `revoke_link` handlers
- `crates/api/api/src/governance/bridge_auth.rs` — bearer-auth helper
- `crates/api/routes/src/governance.rs` — route registration for `/link`, `/link/confirm`, `/link/revoke`
- `crates/server/tests/e2e/actor_app_link.rs` — 5 e2e tests (dual-sig, bad-sig, bad-bearer, pseudonym check, revoke)
- `crates/api/api/Cargo.toml` — `ed25519-dalek` dep addition

**Key ADR constraints (for CR finding evaluation):**
- ADR-015: `actor_pseudonym` UUID in all payloads — NEVER raw person_id/username/email
- ADR-008: `link_confirm` and `revoke_link` MUST call `governance_log::append` before returning success
- Dual-signature: Brehon ed25519 + app countersignature; both via `verify_strict`

## 4. Required reading

- `.claude/rules/branch-manager.md` — file-ownership boundaries, autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (stable `id`, bucket enum, severity enum)

## 5. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- PR number: **197**
- Record poll results in `.claude/runlog/bm-runlog.md`
- Do NOT post comments or submit reviews (those are separate confirm-gated verbs)
- Initial bucket: default `fix-in-pr` for CRITICAL/MAJOR; flag `rebut`/`carry-forward`/`wont-fix` candidates for advisor triage at gate-3 (do not finalize buckets — bm-triage + advisor own that)
- `source: coderabbit` on every finding; stable `id` per finding; `severity` ∈ {critical, major, medium, low, nit}
- `poll_count: 1`, `last_poll_at: <ISO 8601>`
- `counters` block must be regenerated from the finding list (not left at bm-pr shell zeros)
