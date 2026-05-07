---
role: impl-task
phase: v1-SL-b
created: 2026-05-07
---

# Brief — `sl-b-fix-impl-4` — e2e test assertions for cr-7b revoker_pseudonym rename

## 1. Role + dispatch line

`[role:impl-task] sl-b-fix-impl-4 — see .claude/PRPs/briefs/sl-b-fix-impl-4.md`

## 2. Scope

The cr-7b fix (task #133, commit `a771c49e7`) renamed `sponsor_pseudonym` →
`revoker_pseudonym` in the `ENTRY_KIND_ENDORSEMENT_REVOKED` payload in
`crates/api/api_crud/src/governance/revoke_endorsement.rs`.

Two e2e test assertions still reference the old field name and must be updated
to match the new schema:

### Fix A — line 11253 of `crates/server/tests/e2e.rs`

Current:
```rust
assert!(payload["sponsor_pseudonym"].is_string(), "sponsor_pseudonym is a string");
```

Fix:
```rust
assert!(payload["revoker_pseudonym"].is_string(), "revoker_pseudonym is a string");
```

### Fix B — line 3336 of `crates/server/tests/e2e.rs`

Current:
```rust
.any(|p| serde_json::to_string(p).map(|s| s.contains("\"sponsor_pseudonym\"")).unwrap_or(false));
```
And line 3337-3339:
```rust
assert!(
  saw_pseudonym,
  "expected at least one governance_log payload with sponsor_pseudonym"
);
```

Fix (update the contains check and the assertion message):
```rust
.any(|p| serde_json::to_string(p).map(|s| s.contains("\"revoker_pseudonym\"")).unwrap_or(false));
```
```rust
assert!(
  saw_pseudonym,
  "expected at least one governance_log payload with revoker_pseudonym"
);
```

**Context:** Line 11254 (`target_pseudonym`) is correct — `target_pseudonym` IS
emitted by the handler at line 352 of `revoke_endorsement.rs`. Do not touch it.

**Do NOT:**
- Touch `revoke_endorsement.rs` (already fixed)
- Edit any file other than `crates/server/tests/e2e.rs`
- Add new tests or modify test logic beyond the two string literal renames

**Produce:**
- One commit on `phase-v1-SL-b` with the two line fixes
- A `kind: "validate-pending"` DQ entry post-push per Shape G
- Commit subject: `fix(v1-SL-b): update e2e assertions for revoker_pseudonym rename (cr-7b)`

## 3. Required reading

1. `crates/server/tests/e2e.rs` lines 3320–3345 and lines 11240–11265 (the two assertion sites)
2. `crates/api/api_crud/src/governance/revoke_endorsement.rs` lines 344–360 (confirm `target_pseudonym` and `revoker_pseudonym` are both in payload)
3. `.claude/lessons/feedback_clippy_rerun_after_fix.md`

## 3a. Handover from prior cohort

(none — standalone fix task)

## 4. Constraints

- **≤ 2 file edits** — only `crates/server/tests/e2e.rs`; 3 string literal changes in 2 locations
- Run `cargo clippy -p lemmy_api_crud --no-deps -- -D warnings` after editing; fix any new warnings before committing
- After commit + push to `phase-v1-SL-b`, write a `kind: "validate-pending"` DQ entry per Shape G (`.claude/rules/decision-queue.md` validate-pending recipe); commit + push the DQ entry immediately
- Attribution: `answered_by` on the DQ entry must be null; `from: "impl"`
- Do not self-resolve the validate-pending entry — advisor dispatches ci-watcher

## 5. Expected output

- One commit on `phase-v1-SL-b`: `fix(v1-SL-b): update e2e assertions for revoker_pseudonym rename (cr-7b)`
- `kind: "validate-pending"` DQ entry committed + pushed
- Completion summary: which 3 string literals were changed, DQ entry id
