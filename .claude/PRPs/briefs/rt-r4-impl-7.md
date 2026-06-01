# Brief: v1-RT-r4 Task 7 — e2e coverage

## 1. Role + dispatch

```
[role:impl-task] v1-RT-r4 task 7 e2e fixtures for sponsor-gate strategies + admin allowlist — see .claude/PRPs/briefs/rt-r4-impl-7.md
```

## 2. Scope

**One file modified:** `crates/server/tests/e2e.rs`

**Action:** Append a new `mod v1_rt_r4_fixtures { ... }` module at the end of the file (after line 18095, the closing `}` of `v1_rt_r3_fixtures`).

**DO NOT** edit any existing code above the insertion point. The file is ~18k lines; the only edit is one append.

### 2.1 Pre-locate anchor (MANDATORY FIRST STEP)

Before writing ANY code, run:
```bash
grep -n "^}" crates/server/tests/e2e.rs | tail -5
```
Confirm the last `}` is at line 18095 (or re-read the actual last line). Use the verbatim text of the last 3 lines as `old_string` in your Edit call. This is mandatory per `feedback_fix_impl_pre_locate_e2e_anchors.md` — anchor drift in an 18k-line file hangs the editor.

### 2.2 Error shape — Case A (MANDATORY)

Per `feedback_lemmy_error_no_std_error.md` Case A (sibling `v1_rt_r3_fixtures` uses Case A):
- Outer test functions return `LemmyResult<()>`
- All helper functions return `LemmyResult<T>`
- Use `?` propagation throughout — NO `.map_err(|e| Box::new(e))` bridges
- NO `Result<(), Box<dyn std::error::Error>>` anywhere in the module

### 2.3 Story A — Sponsor-gate strategy arms (3 tests)

Module filter: `sponsor_gate_strategies`

Three tests covering the three new strategy arms in `SponsorGateStrategy`:

**Test 1 — `allowlist_arm_passes_for_member`:**
- Set `onboarding.sponsor_gate_strategy` = `"allowlist"` via `admin_set_config`
- Insert a `sponsor_allowlist` row for the sponsoring user via `admin_allowlist_add` HTTP endpoint (POST `/api/v4/governance/admin/sponsor-allowlist/add`)
- Call `create_endorsement` with that user as sponsor
- Assert: `LemmyResult::Ok` (endorsement created)

**Test 2 — `allowlist_arm_denies_non_member`:**
- Set strategy = `"allowlist"`
- Do NOT add sponsor to allowlist
- Call `create_endorsement`
- Assert: returns an error (endorsement denied)

**Test 3 — `reputation_arm_passes_for_can_sponsor_true`:**
- Set strategy = `"reputation"`
- Insert a `reputation_snapshot` row with `can_sponsor = true` for the sponsor, community-scoped to the target community
- Call `create_endorsement`
- Assert: `LemmyResult::Ok`

### 2.4 Story B — Admin allowlist maintenance (3 tests)

Module filter: `admin_sponsor_allowlist`

Three tests covering the add/remove round-trip:

**Test 4 — `add_creates_row_and_emits_log`:**
- POST to `/api/v4/governance/admin/sponsor-allowlist/add` with `{ person_id, community_id: null, note: "test note" }`
- Assert response has `allowlist_id`
- Assert governance_log entry with `entry_kind = "sponsor_allowlist_added"` exists with correct `allowlist_id` in payload

**Test 5 — `remove_deletes_row_and_emits_log`:**
- Add then remove the same row via HTTP endpoints
- Assert governance_log entry with `entry_kind = "sponsor_allowlist_removed"` exists
- Assert `sponsor_allowlist_exists(person_id, None, conn)` returns `false`

**Test 6 — `add_then_endorsement_allowed`:**
- Add person to allowlist
- Set strategy = `"allowlist"`
- Create endorsement
- Assert success

### 2.5 Validate (validate-pending-laptop-e2e)

Write `kind: "validate-pending-laptop-e2e"` DQ entry (id via `bash scripts/brehon/dq-v3-new-entry.sh`):
```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/validate-t7-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/validate-t7-e2e.log || echo E2E_EXIT_NONZERO >> .claude/validate-t7-e2e.log"'
branch: <your worker branch>
phase_task: 7
```
Commit + push the DQ entry. Do NOT run e2e yourself — the laptop runs it.

Also run `cargo check --workspace --features full` and `cargo test --workspace --test e2e --no-run --features full` BEFORE committing to confirm test-compile. Push only on exit 0.

## 3. Required reading

### 3.1 Plan
`.claude/PRPs/plans/v1-RT-r4.plan.md` §13 Task 7 (`:572-600`) and §16a Stories A+B for checkpoint commands.

### 3.2 MIRROR ref — Case A shape (read verbatim before authoring)
`crates/server/tests/e2e.rs` `:17107-17160` — the `v1_rt_r3_fixtures` module header, `use` block, and first test function. Mirror its `LemmyResult<()>` outer + `LemmyResult<T>` helper shape exactly.

### 3.3 Admin endpoint helpers
`crates/api/api/src/governance/admin_sponsor_allowlist.rs` — the `add` and `remove` handler signatures. Call them directly (not via HTTP client) in e2e tests, matching the pattern used for `admin_assign_jury` in `v1_rt_r3_fixtures`.

### 3.4 DB helpers
`crates/db_schema/src/source/governance/sponsor_allowlist.rs` — `sponsor_allowlist_exists`, `sponsor_allowlist_insert`, `sponsor_allowlist_delete` signatures.

### 3.5 Mandatory lessons (e2e.rs edit)
- `feedback_lemmy_error_no_std_error.md` — **Case A shape mandatory** (sibling `v1_rt_r3_fixtures` uses Case A; mirror verbatim).
- `feedback_async_pool_test_pattern.md` — pool setup pattern for e2e tests.
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate verbatim anchors BEFORE editing; file is 18k lines.

## 4. Constraints

- **File-ownership:** modify ONLY `crates/server/tests/e2e.rs` (append only, no edits to existing code). Plus DQ entry commit.
- **Case A (HARD):** `LemmyResult<()>` outer, `LemmyResult<T>` helpers. NO Box<dyn Error> anywhere in this module.
- **Append only:** Insert after the final `}` at line 18095. Never edit above that line.
- **Pre-locate anchor:** verbatim `old_string` from the actual last 3 lines — do NOT guess.
- **Pre-push gate:** `cargo check --workspace --features full` + `cargo test --workspace --test e2e --no-run --features full` both exit 0 before committing.
- **Commit subject:** `feat(rt-r4): e2e fixtures sponsor_gate_strategies + admin_sponsor_allowlist (task 7)`

## 3a. Handover from prior tasks

```yaml
prior_cohort_tasks:
  - task: 3
    commit: 88f4a0d41
    filesCreated: []
    filesModified:
      - crates/api/api_crud/src/governance/create_endorsement.rs
    keyDecisions:
      - SponsorGateStrategy now has: Age, AgeOrSurety, Reputation, Allowlist, Open, Closed, Unknown(String)
      - Allowlist arm calls sponsor_allowlist_exists(sponsor_id, data.community_id, conn)
      - Reputation arm reads reputation_snapshot::can_sponsor scoped by community_id (aliased rs_snapshot)
      - GOTCHA-55a: no _ => catchall; Unknown(String) is final arm
    notes: "Workspace check + clippy clean."
  - task: 4
    commit: f06c38a02
    filesCreated:
      - crates/api/api/src/governance/admin_sponsor_allowlist.rs
    filesModified:
      - crates/api/api/src/governance/mod.rs
    keyDecisions:
      - pub async fn add(...) -> LemmyResult<Json<AddSponsorAllowlistResponse>>
      - pub async fn remove(...) -> LemmyResult<Json<RemoveSponsorAllowlistResponse>>
      - mod.rs: pub mod admin_sponsor_allowlist; (alphabetical)
    notes: "ADR-015 compliant payloads. Workspace check + clippy clean."
  - task: 5
    commit: 9c8176669
    filesCreated: []
    filesModified:
      - crates/api/routes/src/lib.rs
    keyDecisions:
      - Routes: POST /api/v4/governance/admin/sponsor-allowlist/add, /remove
      - Registered after /emergency-remove in admin scope
    notes: "Workspace check + clippy clean."
```
