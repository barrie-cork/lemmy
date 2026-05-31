---
role: impl-task
task: 1
phase: v1-quality-r3c
plan: .claude/PRPs/plans/v1-quality-r3c.plan.md
created: 2026-05-31
authored_by: advisor (canonical brehon-fork / governance-v0 session, Mode B)
base_branch: phase-v1-quality-r3c
---

# impl-task brief — v1-quality-r3c Task 1

**Role:** `[role:impl-task]`
**Task:** T1 — Fix stale `.coderabbit.yaml` endpoint-count rule (Issue #165)
**Phase:** v1-quality-r3c
**Base branch:** `phase-v1-quality-r3c`
**Commit subject:** `fix(coderabbit): scope endpoint-count rule to v0 era — fixes Issue #165`

---

## 1. Role + dispatch line

```
[role:impl-task] v1-quality-r3c T1 — .coderabbit.yaml endpoint-count rule fix — see .claude/PRPs/briefs/v1-quality-r3c-impl-1.md
```

---

## 2. Scope

**Produce:** one commit on `phase-v1-quality-r3c` that edits `.coderabbit.yaml` to remove the stale "EXACTLY 11 endpoints" hard-count assertion.

**Exactly:**
- In `.coderabbit.yaml`, find the path instruction for `crates/api/api_common/src/governance.rs` (lines ~139-148)
- Replace the instruction text: remove "v0 is EXACTLY 11 endpoints per ADR-010" assertion; replace with v1-aware caveat referencing OQ-020
- Keep the "Flag business logic inside DTO impls — DTOs are shape-only" clause intact
- Keep the "RevokeEndorsement DTO is allowed (Phase 3 task 35 exception)" clause intact

**Files to modify (exactly one):**
- `.coderabbit.yaml`

**Do NOT:**
- Touch any file under `crates/`, `migrations/`, `tests/`, `docs/`
- Remove the entire path instruction block — only update the instruction text
- Remove the "DTOs are shape-only" or "flag business logic" clauses

---

## 3. Required reading

### 3a. MIRROR ref

1. **`.coderabbit.yaml` lines 135-165** — read this range to see the full context of the path instruction block being edited, plus adjacent rules.

### 3b. Plan reference

- **`.claude/PRPs/plans/v1-quality-r3c.plan.md` §13 Task 1** — the exact old_string and new_string are specified there. Follow them literally.

---

## 4. Constraints

1. **One file only:** `.coderabbit.yaml` — no other files
2. **Anchor uniqueness:** the old_string `"EXACTLY 11 endpoints"` appears exactly once in `.coderabbit.yaml` (count=1 confirmed). Use this as your Edit anchor.
3. **Preserve adjacent clauses:** the instruction block has multiple sentences. Only the count assertion changes; the DTO-shape and RevokeEndorsement clauses must survive.
4. **Validate after edit:**
   ```bash
   grep -c 'EXACTLY 11' .coderabbit.yaml
   # EXPECT: 0
   grep 'DTO scope check' .coderabbit.yaml
   # EXPECT: shows updated instruction line
   ```
5. **Commit subject exactly:** `fix(coderabbit): scope endpoint-count rule to v0 era — fixes Issue #165`
6. **DQ mid-task push:** if a blocker arises, write `kind: "blocker"` DQ entry via `bash scripts/brehon/dq-v3-new-entry.sh`, commit + push, and stop.
7. **No cargo run required** — this task edits only a YAML config file. No `cargo check`, no `cargo clippy`. The validate commands are `grep`-only.
8. **Write validate-pending-laptop DQ entry after commit + push** (Shape G suspended — per new rule `feedback_validate_pending_laptop_write_then_stop.md`). Write the entry with `commands: ["grep -c 'EXACTLY 11' .coderabbit.yaml"]`, `branch: phase-v1-quality-r3c`, `phase_task: 1`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id and `bash scripts/brehon/dq-v3-append-fragment.sh` to append.

---

## HANDOVER

Brief complete. Dispatch as:

```
[role:impl-task] v1-quality-r3c T1 — .coderabbit.yaml endpoint-count rule fix — see .claude/PRPs/briefs/v1-quality-r3c-impl-1.md
```

Base branch: `phase-v1-quality-r3c`
