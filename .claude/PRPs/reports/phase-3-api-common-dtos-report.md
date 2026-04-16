# Implementation Report: Phase 3 — API Common DTOs

**Plan**: `.claude/PRPs/plans/phase-3-api-common-dtos.plan.md`
**Completed**: 2026-04-16
**Iterations**: 1
**Branch**: `feature/phase-3-api-common-dtos`

## Summary

Defined 12 governance DTO structs in `crates/api/api_common/src/governance.rs`
using Option B (define-in-place, not re-export). All structs use Phase 1
newtypes and enums, follow the upstream derive stack, and compile clean
under `cargo check --workspace` and `cargo clippy --no-deps -D warnings`.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| 31 | Scaffold governance.rs + wire lib.rs + Cargo.toml deps | `85449315a` |
| 32 | Reports + Cases DTOs (4 structs) | `b1bec873a` |
| 33 | Jury DTOs (3 structs) | `689687af7` |
| 34 | Appeals + Public Log DTOs (2 structs) | `af8eafa0f` |
| 35 | Reputation / Trust DTOs (3 structs) | `b99e362b4` |
| 36 | Cargo.toml verification (no-op) | — |
| 37 | Full compile + clippy + workspace check | `852f47db0` |

## Structs Defined (12)

1. `CreateGovernanceReport` — Group A
2. `CreateGovernanceReportResponse` — Group A
3. `GetGovernanceCase` — Group A
4. `ListGovernanceCases` — Group A
5. `SubmitJuryVote` — Group B
6. `AcceptJuryAssignment` — Group B
7. `DeclineJuryAssignment` — Group B
8. `RequestAppeal` — Group C
9. `ListGovernanceModlog` — Group E
10. `GetMyReputation` — Group D
11. `CreateEndorsement` — Group D
12. `RevokeEndorsement` — Group D

Plus `SuccessResponse` (existing upstream type, no action needed).

## Validation Results

| Check | Result | Notes |
|-------|--------|-------|
| cargo check -p lemmy_api_common | PASS | exit 0 |
| cargo check --features ts-rs | SKIP | Pre-existing upstream `DbUrl: TS` failure in lemmy_db_schema; not Phase 3 |
| cargo clippy --no-deps -D warnings | PASS | exit 0 |
| cargo check --workspace | PASS | exit 0 |

## Deviations from Plan

1. **Level 2 validation (ts-rs feature check) skipped.** The `--features ts-rs`
   check fails at `lemmy_db_schema` level (`DbUrl: TS` not satisfied) on both
   `governance-v0` baseline and the feature branch. This is pre-existing upstream
   debt, confirmed by running the same command on `governance-v0` at `c628e09fa`.
   Phase 3 governance code is gated behind `#[cfg_attr(feature = "ts-rs", ...)]`
   and will work correctly once the upstream `DbUrl` issue is resolved.

2. **Cargo.lock committed in task 37** instead of being empty. The ts-rs feature
   check downloaded `ts-rs 12.0.1` and `ts-rs-macros 12.0.1`, updating the
   lockfile. This is a legitimate transitive dependency addition.

## Patterns Discovered

- None new — Phase 3 was a pure-types phase with no query or handler patterns.
