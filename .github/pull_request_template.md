<!--
  Brehon governance-v0 PR template. Delete any row that does not apply.
  Do not remove section headers — they are consumed by the PR reviewer
  (human + CodeRabbit + actions/ai-inference) to structure feedback.
-->

## Phase

- [ ] Phase 1 — Schema + Diesel foundation
- [ ] Phase 2a — `governance_case` + `jury_queue` views
- [ ] Phase 2b — `governance_modlog` views
- [ ] Phase 3 — `api_common` DTOs
- [ ] Phase 4 — First five endpoints + golden-path e2e
- [ ] Phase 5 — Reputation snapshot + remaining six endpoints
- [ ] Phase 6 — Federation outbound + advisory inbound
- [ ] Other (tooling / docs / CI / upstream rebase)

## Summary

<!-- 1–3 sentences on what this PR does and why. Focus on the "why". -->

## Definition of Done

From `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` §3.

- [ ] Tasks in this PR match numbered tasks in the relevant phase section
- [ ] `cargo check --workspace` passes locally (or via the Windows wrapper)
- [ ] `cargo clippy --workspace --tests -- -D warnings` clean
- [ ] `cargo +nightly fmt -- --check` clean
- [ ] Woodpecker is green on this push
- [ ] Integration tests hit a real Postgres (no DB mocks)

## ADR impact matrix

Answer every row. "n/a" is fine where a row is genuinely irrelevant.

| ADR | Touched? | Notes |
|---|---|---|
| ADR-008 append-only `governance_log` / hash chain | no / yes | |
| ADR-013 `CaseStatus::EmergencyRemove` exhaustive match | no / yes | |
| ADR-015 pseudonymised actor IDs / `scrub()` on public strings | no / yes | |
| ADR-010 v0 scope — adds an endpoint or dependency? | no / yes | |
| ADR-006 inbound federation signals advisory-only | no / yes | |
| New ADR added or existing ADR superseded? | no / yes | |

## Validation commands run locally

Paste exit codes or "n/a":

```
cargo check --workspace                exit: ___
cargo clippy --workspace --tests ...   exit: ___
cargo test --test e2e                  exit: ___
cargo +nightly fmt -- --check          exit: ___
```

## Risk / rollback

- Migration reversible via `diesel migration redo`? yes / no / n/a
- If merged and wrong, what's the rollback? (e.g. revert commit, follow-up
  migration, rerun background job, etc.)

## AI review acknowledgement

- [ ] CodeRabbit walkthrough read; each finding acted on or acknowledged
- [ ] `governance-ai-review` workflow comment read (advisory only)
- [ ] `adr-compliance` workflow green, or any red flags discussed in-thread

<!--
  Reviewer checklist (do not fill in — for the maintainer at merge time):
  [ ] Branch protection satisfied
  [ ] Hash-chain verification test still passing
  [ ] No new v2-scope dependencies introduced (Keycloak/OpenFGA/Vault/HSM/KMS)
  [ ] Every user-visible string scrubbed before reaching the governance log
-->
