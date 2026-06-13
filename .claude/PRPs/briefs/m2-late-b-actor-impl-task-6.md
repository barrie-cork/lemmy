---
role: impl-task
task_number: 6
phase: m2-late-b-actor
base_branch: phase-m2-late-b-actor
created: 2026-06-13
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md  # pre-Shape-G
---

# impl-task brief — m2-late-b-actor Task 6: ADD ENTRY_KIND consts + `sign_link_claim` wrapper

**Role:** `[role:impl-task]`
**Phase:** `m2-late-b-actor`
**Task number:** 6 of 13 (`[P]` — cohort-2 alongside Task 5)
**Base branch:** `phase-m2-late-b-actor`
**Authored:** 2026-06-13
**Depends on:** Tasks 3+4 merged. Does NOT depend on Task 5.

---

## 1. Role + dispatch line

```
[role:impl-task] m2-late-b-actor task-6 entry-kind-consts sign-link-claim — see .claude/PRPs/briefs/m2-late-b-actor-impl-task-6.md
```

---

## 2. Scope

**Produce:**
- 2 new `pub const ENTRY_KIND_*` consts in `crates/db_schema/src/source/governance/governance_log.rs`
- `pub fn sign_link_claim(claim_bytes: &[u8]) -> LemmyResult<Vec<u8>>` wrapper in the same file
- `validate-pending-laptop` DQ entry (commit + push)

**Do NOT:**
- Edit the shim at `crates/api/api/src/governance/governance_log.rs` (that is Task 7)
- Edit the registry doc `.claude/rules/governance-log-entry-kind-registry.md` (that is Task 8, advisor-owned)
- Run cargo yourself

---

## 3. Required reading

1. `.claude/PRPs/plans/m2-late-b-actor.plan.md` §"Task 6" + §"Patterns to Mirror / ENTRY_KIND consts" + §"ed25519 SIGN wrapper"
2. `crates/db_schema/src/source/governance/governance_log.rs` lines 250-260 — **MIRROR**: the m2-late-1 ENTRY_KIND block that these consts follow
3. `crates/db_schema/src/source/governance/governance_log.rs` lines 315-360 — **MIRROR**: `load_signing_key` (private fn) + `sign` call shape for the new `sign_link_claim` wrapper
4. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — **MANDATORY**

---

## 4. Constraints

### Implementation

**Step 1: Read governance_log.rs** fully to understand the existing const block and the `load_signing_key` private fn location.

**Step 2: Add 2 ENTRY_KIND consts**

Find the m2-late-1 const block (around line 255-256). After the last entry in that block, add a new comment-delimited block:

```rust
// --- m2-late-b-actor entry kinds (2) ---
pub const ENTRY_KIND_ACTOR_APP_LINK_CREATED: &str = "actor_app_link_created";
pub const ENTRY_KIND_ACTOR_APP_LINK_REVOKED: &str = "actor_app_link_revoked";
```

**GOTCHA — values are lower_snake matching the const tail:**
- `ENTRY_KIND_ACTOR_APP_LINK_CREATED` → `"actor_app_link_created"` (not `"ActorAppLinkCreated"`)
- These are registry-unique values; collision check: `rg -c '"actor_app_link_' governance_log.rs` must return 2 after your edit

**GOTCHA — define-side count:**
- The total `pub const ENTRY_KIND_` count goes from 67 to 69; verify with `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` → must be 69

**Step 3: Add `sign_link_claim` wrapper**

After the ENTRY_KIND consts block (or near the end of the file, after the existing private signing functions), add:

```rust
#[cfg(feature = "full")]
pub fn sign_link_claim(claim_bytes: &[u8]) -> LemmyResult<Vec<u8>> {
  let signing_key = load_signing_key()?;
  Ok(signing_key.sign(claim_bytes).to_bytes().to_vec())
}
```

**GOTCHA — `load_signing_key` stays private:**
- Do NOT add `pub` to `load_signing_key`
- The ONLY new public surface is `sign_link_claim`
- `sign` is called on the `SigningKey` returned by `load_signing_key`; `.to_bytes().to_vec()` converts the `Signature` to `Vec<u8>`

**GOTCHA — feature gate:**
- `#[cfg(feature = "full")]` is mandatory on `sign_link_claim` (it calls `load_signing_key` which is also `#[cfg(feature = "full")]`-gated)
- Without this gate, the non-`full` build will fail

**GOTCHA — LemmyResult return type:**
- `LemmyResult<Vec<u8>>` (uses the crate's `LemmyResult` alias, not `Result<Vec<u8>, LemmyError>`)
- The `?` on `load_signing_key()?` propagates the key-load error

### validate-pending-laptop (MANDATORY)

After committing:

1. Write DQ entry:
   ```json
   {
     "commands": ["cargo check -p lemmy_db_schema"],
     "branch": "phase-m2-late-b-actor",
     "phase_task": 6
   }
   ```
   Use `bash scripts/brehon/dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh --pending`.

2. Commit + push + **STOP**.

---

## 5. Commit

```
feat(db_schema): add ENTRY_KIND_ACTOR_APP_LINK_* consts + sign_link_claim wrapper (task 6)
```

Stage only: `crates/db_schema/src/source/governance/governance_log.rs`

Then DQ entry commit (separate).

---

## 6. DoD

- [ ] `ENTRY_KIND_ACTOR_APP_LINK_CREATED: &str = "actor_app_link_created"` present
- [ ] `ENTRY_KIND_ACTOR_APP_LINK_REVOKED: &str = "actor_app_link_revoked"` present
- [ ] `pub fn sign_link_claim` wrapper present, `#[cfg(feature = "full")]`-gated
- [ ] `load_signing_key` remains private (no `pub` added)
- [ ] `rg -c '^pub const ENTRY_KIND_' governance_log.rs` → 69
- [ ] `validate-pending-laptop` DQ committed + pushed

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-b-actor-impl-task-6
  branch: phase-m2-late-b-actor
  filesModified:
    - crates/db_schema/src/source/governance/governance_log.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "2 new ENTRY_KIND consts; total 67→69"
    - "sign_link_claim pub wrapper; load_signing_key stays private"
    - "Both gated feature=full"
  notes: "Task 6 of 13. Task 7 (shim re-export) follows. Task 8 is advisor-direct."
```
