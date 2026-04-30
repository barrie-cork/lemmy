# [role:impl-task] sweep-2026-04-30 C6b — APUB actor-binding + unused Object import (issue #69)

## 1. Dispatch line

`[role:impl-task] sweep-c6b-actor-binding — see .claude/PRPs/briefs/sweep-2026-04-30-c6b-actor-binding.md`

## 2. Scope

Two findings, both APUB-side:

### Finding 1 (Critical) — Actor-binding check missing in `sanction_notice.rs`

`activitypub_federation` only verifies the HTTP signature against `activity.actor`. The inner governance object's `attributed_to` (the `object.actor` field) is unchecked. This means a federated peer with a valid signature could forge governance objects attributed to a different actor.

**Search for the exact site.** The path candidates per the codebase grep:
- `crates/apub/objects/src/governance/sanction_notice.rs` (the object struct)
- `crates/apub/objects/src/protocol/governance/sanction_notice.rs` (the protocol-side struct)
- `crates/apub/receive/src/governance/sanction_notice.rs` (the receive/verify handler — most likely site)

The fix is approximately:

```rust
if activity.actor() != object.attributed_to {
    return Err(LemmyError::from_message("actor binding mismatch"));
}
```

Place the check at the receive-verification handler, NOT on the object struct. Read the existing verification flow (whichever crate has `verify_*` for governance activities) and add the check before any DB writes.

### Finding 2 (Minor) — Unused `Object` import in `publish_trust_attestation.rs`

The path candidates:
- `crates/apub/send/src/activities/governance/publish_trust_attestation.rs` (likely site)

Remove the unused `Object` import. The fix is mechanical: delete the line.

**Out of scope:**
- Do NOT modify the `activitypub_federation` crate or any upstream-vendored code.
- Do NOT modify the HTTP signature verification.
- Do NOT modify the protocol structs (`*_notice.rs` in `crates/apub/objects/src/protocol/`) unless adding the check requires the field there.
- Do NOT add federation-rejection telemetry beyond a debug log line.

**Boundaries:**
- Edit at most 2 files: the receive-side handler (sanction_notice fix) and `publish_trust_attestation.rs` (import removal).
- Single commit subject: `fix(apub): actor-binding check + unused Object import (closes #69)`.

## 3. Required reading

- **`crates/apub/`** (entire crate tree, structurally) — `tree -L 3 crates/apub/` (or `find crates/apub -name '*.rs' | head -30`) to find the receive handlers
- **The matching receive handler for `sanction_notice`** — most likely `crates/apub/receive/src/governance/sanction_notice.rs` if that path exists; otherwise grep `apub` for `verify` + `sanction_notice`
- **`crates/apub/send/src/activities/governance/publish_trust_attestation.rs`** — remove unused Object import
- **`activitypub_federation` crate's signature-verification path** — read `activitypub_federation::traits::ActivityHandler` or similar trait. Understand what `activity.actor()` returns and how `attributed_to` is exposed on the object.
- **`.claude/lessons/feedback_lemmy_error_no_std_error.md`** — `LemmyError` doesn't implement `std::error::Error`; use `.into()` patterns
- **GitHub issue #69 body**: `gh issue view 69 --repo barrie-cork/lemmy --json body --jq .body`

## 4. Constraints

**HARD FORBIDS:**
- `cargo *` on the worker. Validation OFF-box.
- Editing `crates/api/**` or any non-apub crate.
- Editing `crates/server/tests/e2e.rs`.
- Adding new dependencies to any Cargo.toml.

**Required behaviour:**
- Single commit subject: `fix(apub): actor-binding check + unused Object import (closes #69)`.
- Trailer: `Closes: barrie-cork/lemmy#69`.
- Push branch and exit.
- After push, raise `kind: "validate-pending"` DQ entry with cargo-validate-workspace run id.
- DO NOT open a PR.

**Add a test** if the receive-verification site has a `#[cfg(test)] mod tests` or there's a sibling integration test directory like `crates/apub/receive/tests/`. The test should construct a sanction_notice activity with `activity.actor != object.attributed_to` and assert the verifier returns the new error. If no convenient test surface exists, raise a `kind: "clarify"` DQ asking whether to add a unit test (and where) or rely on review-only verification.

**File-locality:** `crates/apub/receive/...` (or wherever the verify-handler lives) + `crates/apub/send/src/activities/governance/publish_trust_attestation.rs`. Mutually exclusive from C6a (which touches `crates/api/api/src/governance/submit_jury_vote.rs`) and C6c (which touches `crates/api/api/src/governance/redaction.rs`).

**Mid-task DQ push:** if the receive-verification handler structure is unclear (e.g. multiple traits implement verify and it's not obvious where to insert the check), raise `kind: "clarify"` with the candidate sites and stop.
