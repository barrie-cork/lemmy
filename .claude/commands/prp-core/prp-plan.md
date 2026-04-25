---
description: Create a Brehon-aware, Rust-appropriate implementation plan for a v0 phase or feature, keyed to the design docs and IMPLEMENTATION-PLAN-v0.md
argument-hint: <phase number from IMPLEMENTATION-PLAN-v0.md §3 | feature description | path/to/prd>
---

<objective>
Transform "$ARGUMENTS" into a battle-tested Rust implementation plan for the Brehon governance fork, through systematic design-doc reading, codebase exploration, and pattern extraction from Lemmy 1.0-beta.

**Core Principle**: PLAN ONLY — no Rust code written. Produce a context-rich plan document that enables one-pass implementation success.

**Execution Order**: DESIGN DOCS FIRST, THEN CODEBASE, THEN EXTERNAL RESEARCH. Solutions must fit the 15 committed ADRs and the existing Lemmy crate layout before anything new is introduced.

**Agent Strategy**: Use the built-in `Explore` subagent (via the `Agent` tool with `subagent_type="Explore"`) for codebase exploration. Launch up to 3 Explore agents in parallel when scope spans multiple areas.
</objective>

<brehon-context>
**These docs are the authoritative source for this project. Read them before producing any plan. They live in a sibling repo on the same machine, so use the absolute Windows paths:**

P0 (read every time):
- `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` — the phase-by-phase blueprint, cross-cutting requirements, test strategy, Monday-morning checklist
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` — primary backend reference: tables, enums, Diesel structs, view structs, DTOs, route table, handler responsibilities
- `docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md` — the 11-endpoint MVP scope, v0 simplifications, 6-step order, done-definition
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — 15 committed ADRs (hard constraints) and 12 open questions

P1 (read as needed):
- `00-README.md`, `01-vision-and-principles.md`, `02-domain-model.md` — vision, glossary, lifecycles
- `03-architecture.md` — crate layout, plane separation, flow diagrams
- `06-security-and-threat-model.md` — §2.2.1 emergency-remove, §6.1 GDPR, §7 threat table
- `07-operations-and-federation.md` — Docker Compose on one host for v0

P2 (fork-local):
- `CLAUDE.md` at the fork root — pinned upstream SHA, branch info, command list
- `AGPL-NOTICE.md` at the fork root — AGPLv3 source-disclosure obligation

**Do NOT read** `chat1.md`, `chat2.md`, `.docx` files, or the `old-prp-commands/` directory.

**Hard constraints (from [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), do NOT re-litigate):**
- Fork of Lemmy 1.0-beta; use Extism plugin system where it simplifies governance hooks (ADR-012)
- AGPLv3 inherited (ADR-011)
- v0 scope = exactly the 11 endpoints in [05 §2](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md); nothing else
- v0 simplifications in [05 §3](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) are mandatory: 5-juror panels, quorum 3, simple majority, outbound-only federation, local hash chain, reputation-decay stub
- Solo-dev stack: NO Keycloak, NO OpenFGA, NO Vault, NO Kubernetes, NO external log signer, NO blockchain anchoring. Those are v2/v3
- Auth: Lemmy's existing JWT; optional passkey MFA via `webauthn-rs`
- Authz: hardcoded capability checks in Rust reading `reputation_snapshot` flags
- Governance log: `sha2` hash chain via Postgres triggers; `rs_merkle` + `ed25519-dalek` for signing; key in `.env` for v0
- GDPR from day 1: pseudonymised `actor_pseudonym` table mandatory (ADR-015)
- Illegal content from day 1: `CaseStatus::EmergencyRemove` mandatory (ADR-013)
- Federation: content-level with vanilla Lemmy works; governance signals are fork-only AP types (ADR-014)

**If the plan appears to contradict any of these ADRs, STOP. Do not silently fix in the plan body. Move the contradiction to §8 "open questions to escalate" and surface it to the user. ADRs are append-only — changes come through new ADRs, not quiet edits.**
</brehon-context>

<context>
**Project layout** (Lemmy 1.0-beta Rust workspace, forked as `brehon-fork`, working branch `governance-v0`):
- `crates/db_schema/` — Diesel table definitions, Rust types, enums
- `crates/db_views/` — denormalised, permission-aware read models (per-view crates)
- `crates/api/api_common/` — shared request/response DTOs
- `crates/api/api_crud/` — simple CRUD handlers
- `crates/api/api/` — workflow/orchestration handlers
- `crates/api/routes/` — HTTP route registration
- `crates/apub/objects/`, `crates/apub/activities/`, `crates/apub/apub/` — ActivityPub objects, activities, inbox/outbox
- `crates/server/` — composition root: wiring, startup, background jobs (NO business logic)
- `migrations/{timestamp}_name/{up,down}.sql` — Diesel CLI conventions
- `api_tests/` — existing Lemmy integration tests
- `tests/` (to be added for governance e2e) — `tests/e2e.rs` per [IMPLEMENTATION-PLAN-v0.md §5.1](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)

**Expected governance paths** (from [03 §7](docs/brehon-law-inspired-network/03-architecture.md)):
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

**Toolchain:**
- `rust-toolchain.toml` pins the Rust channel (currently `1.94` per upstream)
- `Cargo.toml` is the workspace root — do NOT look for `package.json`
- `diesel.toml` configures `diesel migration run / redo / revert`
- Test databases run in Docker — **use `--user $(id -u):$(id -g)` to avoid root-owned files blocking worktree cleanup**
</context>

<process>

## Phase 0: DETECT — Input Type Resolution

Determine input type:

| Input Pattern | Type | Action |
|---|---|---|
| `Phase N`, `phase 1`, `Step N`, `step 1` | Phase reference to [IMPLEMENTATION-PLAN-v0.md](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) | Read the plan, locate §3 Phase N, extract tasks |
| Ends with `.prd.md` | PRD file | Parse PRD Implementation Phases table, select next pending phase |
| File path that exists | Document | Read and extract feature description |
| Free-form text | Description | Use directly as feature input |
| Empty/blank | Conversation | Use conversation context as input |

### If phase reference:

1. **Read [IMPLEMENTATION-PLAN-v0.md](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) fully.**
2. **Locate §3 Phase N** — extract all numbered tasks for that phase, their definition-of-done, their references back to [04](docs/brehon-law-inspired-network/04-data-model-and-api.md).
3. **Report selection:**
   ```
   SOURCE: IMPLEMENTATION-PLAN-v0.md §3 Phase N
   TASKS: {count}
   DEFINITION OF DONE: {from plan}
   CROSS-CUTTING REQUIREMENTS: hash chain, actor_pseudonym, EmergencyRemove, AGPLv3 (see §4)
   ```

### If PRD or free-form: proceed to Phase 1 with the input as feature description.

**PHASE_0_CHECKPOINT:**
- [ ] Input type determined
- [ ] If phase ref: the relevant §3 Phase is extracted from IMPLEMENTATION-PLAN-v0.md
- [ ] Feature description ready for Phase 1

---

## Phase 1: PARSE — Feature Understanding

EXTRACT from input:
- Core problem / scope
- Which v0 step ([05 §4 Steps 1–6](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md)) this belongs to
- Feature type: `SCHEMA` | `READ_MODEL` | `DTO` | `HANDLER` | `FEDERATION` | `CROSS_CUTTING` | `BUG_FIX`
- Complexity: LOW | MEDIUM | HIGH
- Affected crates (list them)
- Which ADRs in [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) govern this area

**PHASE_1_CHECKPOINT:**
- [ ] Scope is specific and bounded to v0
- [ ] Affected crates identified
- [ ] Relevant ADRs listed (none can be contradicted)
- [ ] Any blocking OQs identified (see [IMPLEMENTATION-PLAN-v0.md §8](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) for the blocking set)

**GATE**: If requirements are AMBIGUOUS or touch an unresolved blocking OQ → STOP and ASK the user before proceeding.

---

## Phase 2: EXPLORE — Codebase Intelligence

**CRITICAL**: Launch up to 3 `Explore` agents in parallel via the Agent tool (`subagent_type="Explore"`), one per distinct area. Use 1 agent for isolated/targeted tasks, 2–3 for phases that span multiple crates.

### Agent brief template

Use the Agent tool with `subagent_type="Explore"` and a prompt shaped like this:

```
You are exploring the brehon-fork Rust workspace (forked from Lemmy 1.0-beta) at
C:\Users\barri\Developer\brehon-fork\. Working branch: governance-v0.

Find code relevant to: {scope from Phase 1}.

LOCATE (return actual file:line references and code snippets):
1. Similar existing Lemmy patterns — analogous tables/handlers/views we can mirror
2. Diesel conventions in use — table definitions in crates/db_schema/src/schema.rs,
   Queryable/Insertable derives in crates/db_schema/src/source/*.rs, enum handling
3. Handler patterns — how existing Lemmy handlers are structured in
   crates/api/api_common/, crates/api/api/, crates/api/api_crud/, and wired in
   crates/api/routes/
4. Error types — custom errors, how they propagate, `LemmyError` / `LemmyResult`
5. Logging patterns — `tracing` macros, span conventions
6. Test patterns — existing integration tests in api_tests/, any tests/ directory,
   fixture setup, how Postgres is spun up for tests
7. Migration conventions — naming (`migrations/{timestamp}_name/{up,down}.sql`),
   enum migrations, triggers if any exist
8. Cargo.toml dependencies — what's already in the workspace vs what would be new
9. Extism plugin system — where `extism` / `extism-convert` are used (ADR-012)
10. ActivityPub conventions — `activitypub_federation` crate usage, `ap_id` naming,
    signature handling

Categorize findings by purpose (schema, views, handlers, routes, tests, apub).
Return ACTUAL code snippets from the codebase, not invented examples. Use
file:line references so the plan can be executed without re-searching.
```

### Merge agent results into a unified discovery table

| Category | File:Lines | Pattern | Rust Snippet |
|---|---|---|---|
| SCHEMA | `crates/db_schema/src/schema.rs:NN-MM` | Diesel table! macro | `table! { ... }` |
| MODEL  | `crates/db_schema/src/source/xx.rs:NN-MM` | `#[derive(Queryable, Selectable, Identifiable)]` | `pub struct Xx { ... }` |
| INSERTFORM | `crates/db_schema/src/source/xx.rs:NN-MM` | `#[derive(Insertable)]` | `pub struct XxInsertForm { ... }` |
| HANDLER | `crates/api/api/src/xx.rs:NN-MM` | `async fn xx(data: Json<T>, context: Data<LemmyContext>)` | ... |
| ROUTE | `crates/api/routes/src/lib.rs:NN-MM` | `.service(...)` wiring | ... |
| ERROR | `crates/utils/src/error.rs:NN-MM` | `LemmyError` / `LemmyResult<T>` | ... |
| TEST | `api_tests/src/xx.ts:NN-MM` | fixture + assertion pattern | ... |
| APUB | `crates/apub/.../xx.rs:NN-MM` | `Object` + `Activity` traits | ... |

**PHASE_2_CHECKPOINT:**
- [ ] Explore agents launched (parallel where scope spans areas) and completed
- [ ] At least 3 similar Lemmy implementations found with file:line refs
- [ ] Code snippets are ACTUAL (copy-pasted, not invented)
- [ ] Integration points mapped (which existing Lemmy code will be called)
- [ ] Relevant dependencies cataloged with versions from `Cargo.toml`

---

## Phase 3: RESEARCH — External Documentation (only when needed)

**Only after Phase 2** and only when external research is actually required (e.g. a new crate, an unfamiliar Diesel feature, an ActivityPub spec detail). Use `WebSearch` + `WebFetch` directly rather than spawning an agent — for a solo-dev v0 the extra ceremony of a research agent is overhead.

Focus on:
- Crate docs.rs pages for `diesel`, `diesel-derive-enum`, `activitypub_federation`, `extism`, `sha2`, `ed25519-dalek`, `rs_merkle`, `webauthn-rs` — match versions to `Cargo.toml`
- Diesel migration patterns, especially Postgres enums and triggers
- Lemmy upstream docs / issue tracker for 1.0-beta API churn

FORMAT findings into plan references:

```markdown
- [Crate Docs v{version}](https://docs.rs/crate/version/...)
  - KEY_INSIGHT: {what affects implementation}
  - APPLIES_TO: {which task/file}
  - GOTCHA: {pitfall and mitigation}
```

**PHASE_3_CHECKPOINT** (if research was needed):
- [ ] URLs include specific section anchors (not just homepage)
- [ ] Versions match `Cargo.toml`
- [ ] Gotchas documented with mitigation strategies
- [ ] No conflicting patterns between external docs and codebase

---

## Phase 4: DESIGN — UX / Flow Transformation

For Brehon, "UX" usually means **API flow** (there's no v0 frontend). Build an ASCII diagram of the flow before and after.

Example (adjust to the phase):

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                              BEFORE STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║   Lemmy user POSTs content → standard Lemmy report flow → Lemmy modlog        ║
║                                                                               ║
║   DATA_FLOW: report → lemmy modlog → direct moderator action                  ║
║   PAIN_POINT: no procedural jury; moderator-only decisions                    ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝

╔═══════════════════════════════════════════════════════════════════════════════╗
║                               AFTER STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║   User POST /api/v4/governance/report → moderation_case upsert →              ║
║   ThresholdMet → admin_assign_jury → 5 jurors → votes ≥ 3 →                   ║
║   Decided → Sanction row → PublicCaseLog entry → modlog visible               ║
║                                                                               ║
║   DATA_FLOW: report → case → jury → sanction → public log → governance log    ║
║   VALUE_ADD: procedural moderation without a permanent moderator class        ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

**DOCUMENT interaction changes:**

| Endpoint / Entrypoint | Before | After | Impact |
|---|---|---|---|
| `POST /api/v4/governance/report` | didn't exist | opens/appends to a `moderation_case` | reports become case threshold contributions |
| `GET /api/v4/governance/modlog` | didn't exist | redacted public case log | members can inspect every decision |

**PHASE_4_CHECKPOINT:**
- [ ] Before/After flow described with data flow
- [ ] Each affected endpoint listed
- [ ] Redaction / pseudonym touchpoints noted (cross-cutting §4 of the implementation plan)

---

## Phase 5: ARCHITECT — Strategic Design

For complex phases, launch one more `Explore` agent focused on how existing Lemmy architecture behaves at the integration points identified in Phase 2.

**Then analyse:**
- ARCHITECTURE_FIT: does this respect the plane separation in [03 §4](docs/brehon-law-inspired-network/03-architecture.md)? Is governance code in the governance crate paths, not in `crates/server/`?
- EXECUTION_ORDER: dependency order of tasks (schema → model → view → DTO → handler → route)
- FAILURE_MODES: transaction boundaries, orphaned log entries, redaction bypasses, enum match holes
- PERFORMANCE: n+1 queries, unindexed reads, unbounded joins
- SECURITY: hash-chain integrity, pseudonym leakage, authz bypass via missing capability checks
- GDPR: any new write to `governance_log` must use `actor_pseudonym` and be redacted
- CROSS-CUTTING: does this touch the hash chain, redaction service, or `EmergencyRemove` variant? (If yes, wire through [IMPLEMENTATION-PLAN-v0.md §4](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) cross-cutting helpers.)

**DECIDE and document:**

```markdown
APPROACH_CHOSEN: {description}
RATIONALE: {why this over alternatives, referencing Lemmy patterns and ADRs}

ALTERNATIVES_REJECTED:
- {Alt 1}: rejected because {reason}
- {Alt 2}: rejected because {reason}

NOT_BUILDING (explicit v0 scope limits):
- {Item 1 — deferred to v1/v2/v3 per ADR-010}
- {Item 2 — out of scope}
```

**PHASE_5_CHECKPOINT:**
- [ ] Approach aligns with the ADRs (no contradictions)
- [ ] Dependencies ordered (schema → models → views → DTOs → handlers → routes → wiring)
- [ ] Edge cases identified with mitigation
- [ ] Scope boundaries are explicit and cite v0 simplifications

---

## Phase 6: GENERATE — Implementation Plan File

**OUTPUT_PATH**: `.claude/PRPs/plans/{kebab-case-phase-or-feature-name}.plan.md`

Create the directory if needed: `mkdir -p .claude/PRPs/plans`

### PLAN_STRUCTURE

```markdown
# Plan: {Phase / Feature Name}

## Summary
{One paragraph: what this phase/feature delivers and the high-level approach}

## Source
- [IMPLEMENTATION-PLAN-v0.md](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) §3 Phase N (or PRD path)
- Relevant [04](docs/brehon-law-inspired-network/04-data-model-and-api.md) sections: §N, §M
- Relevant ADRs from [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md): ADR-XXX, ADR-YYY

## Problem Statement
{Specific, testable problem this phase solves}

## Solution Statement
{Architecture overview — which crates touched, which Lemmy patterns mirrored}

## Metadata

| Field | Value |
|---|---|
| Type | SCHEMA / READ_MODEL / DTO / HANDLER / FEDERATION / CROSS_CUTTING / BUG_FIX |
| Complexity | LOW / MEDIUM / HIGH |
| Crates Affected | `db_schema`, `db_views/xx`, ... |
| v0 Step | Step N from [05 §4](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) |
| Dependencies | {prior phases that must be complete} |
| Estimated Tasks | {count} |

---

## Flow Design

### Before State
{ASCII diagram}

### After State
{ASCII diagram}

### Endpoint Changes

| Endpoint | Before | After |
|---|---|---|
| ... | ... | ... |

---

## Mandatory Reading (implementation agent MUST read before starting)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `crates/db_schema/src/source/xx.rs` | NN-MM | Diesel model pattern to MIRROR |
| P0 | `docs/brehon-law-inspired-network/04-data-model-and-api.md` | §N | Authoritative field definitions |
| P1 | `api_tests/src/xx.ts` | all | Lemmy's existing integration-test style |

**External Documentation:**
| Source | Version | Section | Why |
|---|---|---|---|
| [diesel](https://docs.rs/diesel/{v}) | {v} | {section} | {reason} |

---

## Patterns to Mirror

**DIESEL_TABLE_MACRO:**
```rust
// SOURCE: crates/db_schema/src/schema.rs:NN-MM
// COPY THIS PATTERN:
{actual snippet from codebase}
```

**QUERYABLE_MODEL:**
```rust
// SOURCE: crates/db_schema/src/source/xx.rs:NN-MM
// COPY THIS PATTERN:
{actual snippet}
```

**INSERT_FORM:**
```rust
// SOURCE: crates/db_schema/src/source/xx.rs:NN-MM
// COPY THIS PATTERN:
{actual snippet}
```

**HANDLER:**
```rust
// SOURCE: crates/api/api/src/xx.rs:NN-MM
// COPY THIS PATTERN:
{actual snippet}
```

**ROUTE_REGISTRATION:**
```rust
// SOURCE: crates/api/routes/src/lib.rs:NN-MM
// COPY THIS PATTERN:
{actual snippet}
```

**TEST_PATTERN:**
```rust
// SOURCE: api_tests/... or tests/e2e.rs:NN-MM
// COPY THIS PATTERN:
{actual snippet}
```

---

## Files to Change

| File | Action | Justification |
|---|---|---|
| `migrations/{ts}_add_xx/up.sql` | CREATE | New table(s) per [04 §1](docs/brehon-law-inspired-network/04-data-model-and-api.md) |
| `migrations/{ts}_add_xx/down.sql` | CREATE | Clean rollback |
| `crates/db_schema/src/schema.rs` | UPDATE | Regen by `diesel print-schema` |
| `crates/db_schema/src/source/governance/xx.rs` | CREATE | Diesel model + InsertForm |
| `crates/db_schema/src/source/governance/mod.rs` | UPDATE | Export new module |
| `crates/api/api_common/src/governance.rs` | UPDATE | Add DTOs |
| `crates/api/api/src/governance/xx.rs` | CREATE | Handler |
| `crates/api/routes/src/governance.rs` | UPDATE | Wire route |
| `tests/e2e.rs` | UPDATE | Add integration test |

---

## NOT Building (v0 scope limits)

- {Item 1 — deferred to v1/v2/v3 per [99 ADR-010](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)}
- {Item 2}

---

## Step-by-Step Tasks

Execute in order. One commit per task. Each task has a MIRROR reference, an exact file path, and a validation command.

> **Skill triggers in task wording (principle, not rule).** When a task is *shaped like* one of the project's user-invocable implementation skills, name the skill in the task body — but only as a "prefer when…" signal, never as a mandate. Each skill's own SKILL.md is authoritative for the conditions; plans defer to it. The principles below explain what each skill buys you so the impl session can decide.
>
> - **`/test-write`** enforces the e2e harness's `LemmyResult<()>` + pseudonymisation + no-`unwrap` discipline. Mention it in tasks that add new e2e cases under `crates/server/tests/e2e.rs` needing fixture scaffolding. Don't mention it for inline assertion extensions or `#[cfg(test)] mod tests` unit cases.
> - **`/edit-mechanical`** makes rg-enumerate-first a precondition for repeat-pattern edits (R5.1-class bug prevention). Mention it in tasks that propagate a single pattern across N call sites: struct-field additions with `derive(Default)`, enum-variant renames, lint-fix attribute applications. Don't mention it for one-of-a-kind edits or changes needing type/borrow reasoning beyond the rename surface.
> - **`/cargo-validate`** keeps cargo's exit code intact and the conversation context lean. Mention it in **VALIDATE** lines where the cargo run is the gating signal for the task (per-task DoD checks). Don't mention it for incidental scratch runs or background sweeps.
>
> When in doubt, omit the skill mention — the impl session can still invoke the skill on its own judgment. A spurious "use `/test-write` here" in a one-line assertion task is worse than no mention.

### Task 1: CREATE `migrations/{ts}_add_xx/up.sql` + `down.sql`
- **ACTION**: Generate migration via `diesel migration generate add_xx`, then write the SQL
- **IMPLEMENT**: Table + indexes per [04 §1](docs/brehon-law-inspired-network/04-data-model-and-api.md). Enum types defined in the earlier enums migration
- **MIRROR**: `migrations/{existing_example}/up.sql` — follow naming and index style
- **GOTCHA**: Enum types must exist before the tables that reference them (order the migration timestamps)
- **GOTCHA**: `down.sql` must drop in reverse dependency order
- **VALIDATE**: `diesel migration run && diesel migration redo` — both directions work, no errors

### Task 2: CREATE Diesel model in `crates/db_schema/src/source/governance/xx.rs`
- **ACTION**: Write the `#[derive(Queryable, Selectable, Identifiable)]` struct and the `Insertable` form
- **IMPLEMENT**: Exact fields from [04 §3](docs/brehon-law-inspired-network/04-data-model-and-api.md)
- **MIRROR**: `crates/db_schema/src/source/{similar_lemmy_struct}.rs`
- **IMPORTS**: `use crate::schema::xx;`, `use diesel::prelude::*;`, `use chrono::{DateTime, Utc};`, relevant enum imports
- **GOTCHA**: `#[diesel(table_name = xx)]` must match `schema.rs` exactly; use `i32` for ids unless the table uses `serial8`/`BigInt`
- **GOTCHA**: **Every match on `CaseStatus` must cover `EmergencyRemove`** per [ADR-013](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) — use exhaustive match, no `_ =>` fallthrough
- **VALIDATE**: `cargo check -p lemmy_db_schema`

### Task 3: EXPORT from `crates/db_schema/src/source/governance/mod.rs`
- **ACTION**: `pub mod xx; pub use xx::*;`
- **VALIDATE**: `cargo check -p lemmy_db_schema`

### Task 4: ADD DTOs to `crates/api/api_common/src/governance.rs`
- **ACTION**: Add request/response structs per [04 §5](docs/brehon-law-inspired-network/04-data-model-and-api.md)
- **IMPLEMENT**: `#[derive(Debug, Clone, Serialize, Deserialize)]` + `ts_rs` derive if the workspace uses it
- **MIRROR**: `crates/api/api_common/src/{similar_module}.rs`
- **VALIDATE**: `cargo check -p lemmy_api_common`

### Task 5: CREATE handler `crates/api/api/src/governance/xx.rs` (or `api_crud/src/governance/xx.rs`)
- **ACTION**: Implement the async handler per [04 §6](docs/brehon-law-inspired-network/04-data-model-and-api.md)
- **IMPLEMENT**: Validate input, call into the db layer, return the DTO
- **MIRROR**: `crates/api/api/src/{similar_handler}.rs`
- **GOTCHA**: **Every governance write must call the cross-cutting `governance_log::append(...)` helper before the user response returns** (cross-cutting §4.1 of [IMPLEMENTATION-PLAN-v0.md](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md))
- **GOTCHA**: **Any string written to `public_case_log.summary`, `public_case_log.rationale_redacted`, or `governance_log.payload` must pass through `redaction::scrub(...)`** — this is a hard GDPR requirement ([ADR-015](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md))
- **GOTCHA**: Use `actor_pseudonym::get_or_create(person_id)` — never write `person_id`, usernames, or emails into the governance log
- **VALIDATE**: `cargo check -p lemmy_api` (or `lemmy_api_crud`)

### Task 6: WIRE route in `crates/api/routes/src/governance.rs`
- **ACTION**: Register the new endpoint under `/api/v4/governance/...`
- **MIRROR**: existing route registrations in `crates/api/routes/src/lib.rs`
- **VALIDATE**: `cargo check -p lemmy_routes`

### Task 7: ADD integration test in `tests/e2e.rs`
- **ACTION**: Extend the e2e harness with a test that seeds the DB, calls the endpoint, and asserts the row/effect
- **MIRROR**: existing tests in `tests/e2e.rs` (once Phase 1 creates it) or `api_tests/` for reference
- **GOTCHA**: Use `docker run --user $(id -u):$(id -g) postgres:16` — otherwise root-owned files block worktree cleanup
- **VALIDATE**: `cargo test --test e2e {test_name}`

---

## Testing Strategy

Per [IMPLEMENTATION-PLAN-v0.md §5](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): **integration-only** for v0, no unit tests until something breaks twice. All tests live in `tests/e2e.rs`.

### Tests to Add

| Test Name | What It Validates |
|---|---|
| `{test_name_1}` | {happy path} |
| `{test_name_2}` | {edge case or error path} |

### Edge Cases

- [ ] Hash-chain integrity still holds after the new write path
- [ ] `actor_pseudonym` is generated (or reused) correctly
- [ ] `EmergencyRemove` branch (if touched) is exhaustively matched
- [ ] Redaction strips identifiers from any string that reaches the log
- [ ] Rollback migration (`diesel migration redo`) works
- [ ] {feature-specific edge case}

---

## Validation Commands

Use these exact commands — do NOT substitute npm/pnpm/etc. This is a Rust project.

> **Wrapping cargo invocations.** The cargo command lines below are the *canonical shape* of what gets run; the impl session will typically execute them through `/cargo-validate`, which captures full output to a log file and returns exit code + tail-20. The wrapping serves two concerns documented in `.claude/rules/`: it preserves cargo's exit code (which inline pipes to `tail`/`head`/`grep` mask) and it keeps cargo log tails out of the conversation token budget. Plans don't need to spell out the wrapper invocation in every Level — the discipline lives in the skill, and the cargo command lines below are what gets passed to it.

### Level 1: STATIC_ANALYSIS

```bash
cargo check --workspace
cargo clippy --workspace -- -D warnings
```

**EXPECT**: Exit 0, zero errors, zero warnings

### Level 2: INTEGRATION_TESTS

```bash
cargo test --test e2e {test_pattern}
```

**EXPECT**: All tests pass. First run will pull `postgres:16` — allow time

### Level 3: FULL_BUILD

```bash
cargo build --workspace
```

**EXPECT**: Exit 0, no errors

### Level 4: MIGRATION_VALIDATION (if schema changed)

```bash
diesel migration run
diesel migration redo   # verifies up+down round-trip
psql -h localhost -U lemmy -d lemmy_test -c '\d {new_table}'
```

**EXPECT**: Round-trip works; new table shape matches [04](docs/brehon-law-inspired-network/04-data-model-and-api.md)

### Level 5: CROSS_CUTTING_VERIFICATION (if touching log or pseudonyms)

- [ ] Every new write path calls `governance_log::append(...)`
- [ ] Every string that reaches the log passed through `redaction::scrub(...)`
- [ ] No direct `person_id` or username written to the log
- [ ] Hash-chain test (`governance_log_hash_chain_holds`) still passes

### Level 6: MANUAL_VALIDATION

{Step-by-step manual curl / psql commands}

---

## Acceptance Criteria

- [ ] All specified functionality implemented
- [ ] Level 1–3 validation commands pass with exit 0
- [ ] Integration tests cover the happy path and the Phase-relevant edge cases
- [ ] Code mirrors existing Lemmy patterns (naming, file layout, error propagation)
- [ ] No new `cargo clippy` warnings introduced
- [ ] No contradictions with the 15 ADRs in [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)
- [ ] Cross-cutting requirements ([IMPLEMENTATION-PLAN-v0.md §4](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)) respected

---

## Completion Checklist

- [ ] All tasks completed in dependency order
- [ ] Each task validated immediately after completion (Level 1 after every change)
- [ ] Level 1: `cargo check --workspace` + `cargo clippy` pass
- [ ] Level 2: integration tests pass
- [ ] Level 3: `cargo build --workspace` succeeds
- [ ] Level 4: migration round-trip works (if schema changed)
- [ ] Level 5: cross-cutting verification passes (if log or pseudonyms touched)
- [ ] All acceptance criteria met

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Upstream Lemmy 1.0-beta rebase breaks patterns | MED | MED | Pin to `upstream/main` SHA in `CLAUDE.md`; rebase weekly |
| Redaction bypass via direct log write | LOW | HIGH | Single `governance_log::append` wrapper; reject PRs that insert into log table directly |
| Hash-chain trigger + signature update brittleness | LOW | HIGH | Fallback = side-table for signatures (documented in [IMPLEMENTATION-PLAN-v0.md §7.1](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)) |
| {phase-specific risk} | {L/M/H} | {L/M/H} | {mitigation} |

---

## Notes

{Additional context, design decisions, trade-offs}
```

</process>

<output>
**OUTPUT_FILE**: `.claude/PRPs/plans/{kebab-case-phase-or-feature-name}.plan.md`

**If input was a phase reference**, also update [IMPLEMENTATION-PLAN-v0.md](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) with a pointer to the new plan? **NO.** That file lives in a sibling repo and is authoritative — do not edit it from this fork. Link to it instead.

**REPORT_TO_USER**:

```markdown
## Plan Created

**File**: `.claude/PRPs/plans/{name}.plan.md`

**Source**: {IMPLEMENTATION-PLAN-v0.md §3 Phase N | PRD | free-form}
**v0 Step**: {from [05 §4](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md)}
**Complexity**: {LOW/MEDIUM/HIGH}

### Scope

- {N} files to CREATE
- {M} files to UPDATE
- {K} tasks

### Key Patterns Discovered

- {Pattern 1 from Explore agent with file:line}
- {Pattern 2}

### ADRs Governing This Phase

- {ADR-NNN from [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)}

### Cross-Cutting Touches

- {Hash chain / actor_pseudonym / EmergencyRemove / AGPL notice — which apply}

### Blocking OQs

{List any OQ from [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) that blocks execution, or "None"}

### Top Risk

- {Primary risk}: {mitigation}

### Confidence Score

{1-10}/10 for one-pass implementation success
- {Rationale}

**Next Step**: `/prp-implement .claude/PRPs/plans/{name}.plan.md`
```

</output>

<verification>
**FINAL_VALIDATION before saving plan:**

**DESIGN_DOC_COMPLIANCE:**
- [ ] Every reference to [04](docs/brehon-law-inspired-network/04-data-model-and-api.md), [05](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md), and [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) uses the correct path
- [ ] No contradictions with the 15 ADRs
- [ ] No v1/v2/v3 scope has leaked in
- [ ] Solo-dev stack respected (no Keycloak, OPA, Vault, K8s, external signer, blockchain)

**CONTEXT_COMPLETENESS:**
- [ ] All patterns documented with `file:line` refs from actual Lemmy code
- [ ] `Cargo.toml` versions cited where external research was needed
- [ ] Integration points mapped with specific crate/function paths
- [ ] Gotchas captured with mitigation
- [ ] Every task has at least one executable validation command

**IMPLEMENTATION_READINESS:**
- [ ] Tasks ordered by Rust dependency (schema → models → DTOs → handlers → routes)
- [ ] Each task is atomic and independently testable
- [ ] No placeholders — all content is specific and actionable
- [ ] Pattern references include actual Rust snippets (copy-pasted from the workspace)

**PATTERN_FAITHFULNESS:**
- [ ] Every new file mirrors existing Lemmy style
- [ ] No unnecessary abstractions
- [ ] Naming follows Lemmy conventions (snake_case, `lemmy_xxx` crate prefix, etc.)
- [ ] Error handling uses `LemmyError` / `LemmyResult<T>` (or whatever 1.0-beta uses)

**CROSS_CUTTING_COVERAGE** (if applicable):
- [ ] Hash-chain log append path is called for any governance write
- [ ] `actor_pseudonym::get_or_create` is used for any log write
- [ ] Redaction service is called for any string that reaches the log
- [ ] `CaseStatus::EmergencyRemove` is exhaustively matched where `CaseStatus` appears
- [ ] AGPLv3 / source-disclosure touched (only if release artefacts are being produced)

**NO_PRIOR_KNOWLEDGE_TEST**: Could an agent unfamiliar with Brehon implement this using ONLY the plan + the design docs it references?
</verification>

<success_criteria>
**CONTEXT_COMPLETE**: All patterns, gotchas, integration points documented from actual Lemmy code and the Brehon design docs
**ADR_COMPLIANT**: No contradictions with the 15 ADRs in [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)
**IMPLEMENTATION_READY**: Tasks executable top-to-bottom without re-research
**PATTERN_FAITHFUL**: Every new file mirrors existing Lemmy crate style
**VALIDATION_DEFINED**: Every task has an executable `cargo` or `diesel` command
**CROSS_CUTTING_RESPECTED**: Hash chain, pseudonyms, redaction, `EmergencyRemove` wired through where applicable
**ONE_PASS_TARGET**: Confidence score 8+ for first-attempt success
</success_criteria>
</content>
</invoke>