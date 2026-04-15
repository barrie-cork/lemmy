# Phase 3 rubric — API common DTOs

Phase 3 adds request / response types in `crates/api/api_common/src/governance.rs`.
This is the highest-risk phase for **scope creep** — DTOs are cheap to add
so "one more endpoint" is tempting.

Focus areas beyond the ADR rubric:

- **Scope: exactly the 7 DTO groups** listed in `IMPLEMENTATION-PLAN-v0.md`
  §3 Phase 3 tasks 31–37. They map to the 11 v0 endpoints from
  `05-mvp-and-delivery-plan.md` §2. Flag any new DTO that does not map to
  one of those endpoints.

- **`RevokeEndorsement` DTO is allowed** (task 35) even though the endpoint
  itself is deferred to v1 per ADR-010 — this is an explicit exception.
  Do not flag it as scope creep.

- **Every DTO derives `Serialize` + `Deserialize`.** If `ts-rs` is in use
  elsewhere in `api_common`, derive `TS` too — match the existing pattern in
  `crates/api/api_common/src/` for other DTO modules (e.g. `person.rs`,
  `community.rs`).

- **Enums come from `db_schema`, not redefined.** Any DTO that references
  `CaseStatus`, `CaseTargetType`, `JuryDecision`, `SanctionAction`,
  `SanctionScope`, or `ReputationDimension` must pull the enum from
  `lemmy_db_schema::source::governance::*`. Flag duplicate enum definitions
  in `api_common`.

- **No business logic in `api_common`.** DTOs are shape-only types. Flag any
  method beyond `new()`, `Default`, simple builders, or `From`/`TryFrom`
  conversions.

- **Naming convention.** DTO structs match the endpoint name verb-object
  form: `CreateGovernanceReport` / `CreateGovernanceReportResponse`,
  `SubmitJuryVote` / `SubmitJuryVoteResponse`, etc. Flag inconsistent
  naming within the same file.
