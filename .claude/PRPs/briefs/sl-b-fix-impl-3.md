---
role: impl-task
phase: v1-SL-b
created: 2026-05-07
---

# Brief — `sl-b-fix-impl-3` — CR fix-in-PR cr-7 (governance-log payload schema)

## 1. Role + dispatch line

`[role:impl-task] sl-b-fix-impl-3 — see .claude/PRPs/briefs/sl-b-fix-impl-3.md`

## 2. Scope

**Fix CR finding cr-7** on PR #119: governance-log payloads in
`revoke_endorsement.rs` don't fully match the v1 schema.

**Two sub-items:**

### cr-7a — `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` missing `"version": 1`

In `crates/api/api_crud/src/governance/revoke_endorsement.rs` around
line 319-325, the `governance_log::append` call for
`ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` emits:
```json
{"case_id": ..., "escaped_at": ..., "reason": ..., "actor_pseudonym": ..., "endorsement_id": ...}
```
It is **missing `"version": 1`**. Add it as the first key.

Registry entry (`governance-log-entry-kind-registry.md`) specifies payload:
`{case_id, escaped_at, reason, actor_pseudonym, endorsement_id|restoration_id}`
The `version: 1` marker is the v1 schema convention (all other v1-SL-b
payloads include it — see `ENTRY_KIND_ENDORSEMENT_REVOKED` at line 346).

### cr-7b — `ENTRY_KIND_ENDORSEMENT_REVOKED` uses `sponsor_pseudonym` not `revoker_pseudonym`

The registry entry for `ENTRY_KIND_ENDORSEMENT_REVOKED` specifies:
`{endorsement_id, revoker_pseudonym, revoked_at, sponsored_id, reason, liability_chain_severed_for_cases}`

Current code at line 345-353 uses `sponsor_pseudonym` (not `revoker_pseudonym`),
and omits `revoked_at` and `sponsored_id`. Fix:
- Rename `sponsor_pseudonym` → `revoker_pseudonym` in the payload
- Add `"revoked_at": now` to the payload
- Add `"sponsored_id": row.to_person_id.0` to the payload

**Note on cr-6:** CR finding cr-6 (`majority_revocation` baseline) is
already handled by a documented `TODO(v1-SL-c)` comment at lines 287-297.
The code intentionally falls through to `any_revocation` behaviour with an
advisor-relay note in the runlog. **Do not touch the `majority_revocation`
arm** — this is a pre-agreed deferral to v1-SL-c. Bucket cr-6 as `rebut`
in any DQ log entry.

**Do NOT:**
- Touch `majority_revocation` arm or step 6/7 escape logic
- Edit any file outside `crates/api/api_crud/src/governance/revoke_endorsement.rs`
- Add new governance log entry kinds
- Modify migrations, tests, or Cargo files

**Produce:**
- One commit on `phase-v1-SL-b` with the two payload fixes
- A `kind: "validate-pending"` DQ entry post-push per Shape G
- Commit subject: `fix(v1-SL-b): cr-7 governance-log payload schema — version + field names`

## 3. Required reading

1. `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` — active plan (§4 watchpoints, §15 DoD)
2. `.claude/rules/governance-log-entry-kind-registry.md` — payload schema for ENTRY_KIND_ENDORSEMENT_REVOKED + ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED
3. `crates/api/api_crud/src/governance/revoke_endorsement.rs` — the file to fix (full read before editing)
4. `.claude/lessons/feedback_multi_write_handlers_need_transactions.md`
5. `.claude/lessons/feedback_clippy_rerun_after_fix.md`

## 3a. Handover from prior cohort

(none — this is a standalone fix task, not a cohort member)

## 4. Constraints

- **≤ 3 file edits** — this is a narrow CR fix, not a refactor
- Run `cargo clippy -p lemmy_api_crud --no-deps -- -D warnings` on the EliteDesk worktree after editing; fix any new warnings before committing
- After commit + push to `phase-v1-SL-b`, write a `kind: "validate-pending"` DQ entry per Shape G (`.claude/rules/decision-queue.md` validate-pending recipe); commit + push the DQ entry immediately
- Attribution: `answered_by` on the DQ entry must be null; `from: "impl"`
- Do not self-resolve the validate-pending entry — advisor dispatches ci-watcher

## 5. Expected output

- One commit on `phase-v1-SL-b`: `fix(v1-SL-b): cr-7 governance-log payload schema — version + field names`
- `kind: "validate-pending"` DQ entry committed + pushed
- Completion summary: which two payload fields were fixed, confirm cr-6 was rebutted (not touched), DQ entry id
