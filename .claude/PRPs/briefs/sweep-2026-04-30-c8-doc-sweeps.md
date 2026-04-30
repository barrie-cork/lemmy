# [role:impl-task] sweep-2026-04-30 C8 — doc-only sweeps (issues #51, #50, #40, #37)

## 1. Dispatch line

`[role:impl-task] sweep-c8-doc-sweeps — see .claude/PRPs/briefs/sweep-2026-04-30-c8-doc-sweeps.md`

## 2. Scope

Four doc/YAML edits bundled because each is small and none requires compilation:

### #51 — DQ-6.7 wording stale

`.claude/decision-queue.json` entry id 38 (DQ-6.7). The current `question` and `answer` text describe `build_local_sanction_notice_plan` as reading via a fresh pool connection (option a). The shipped code uses the **in-flight `AsyncPgConnection`** passed by `send_local_sanction_notice` (option b — the planner reads inside the outer transaction). Update the `answer` field to:

- State that the wrapper accepts the in-flight `AsyncPgConnection`.
- The plan-build reads the just-inserted sanction row inside the outer tx scope.
- Reference commit `41f1d0379` if relevant.

The shipped behavior is also documented in the **resolved DQ #38's existing `answer` field** at length (read it for context). If that resolved entry already accurately describes option (b), the issue may be a request to update the **`question`/options text** (which describes option a as the brief). Decide based on reading: if the answer is already correct, edit the `question` text to remove the misleading "fresh pool conn" framing. If the answer is also wrong, fix both.

### #50 — phase-6-federation.plan.md MD040 + stale enum names

`.claude/PRPs/plans/phase-6-federation.plan.md` (1218 lines) has two minor doc bugs:

1. **Bare fenced code blocks (MD040)** at lines 97-127 ("AFTER" ASCII diagram) and lines 848-877 (second ASCII block). Fix: add `text` or `ascii` language tag to each block's opening ` ``` ` fence.
2. **Stale enum type names** at lines 480-493: `attestation_type_enum`, `sanction_action_enum`, `sanction_scope_enum` reference the OLD enum naming (with `_enum` suffix). Update to current names — verify by `grep -n "create type\|CREATE TYPE" migrations/` to find the actual enum names.

### #40 — admin-backstop carve-out doc

`crates/api/api_common/src/governance.rs:287` (module header) has the file-level coding guideline: "v0 is EXACTLY 11 endpoints per ADR-010". Currently the only carve-out documented is `RevokeEndorsement`. Update the module header / carve-out comment to also document:

- `AdminReputationStats` → `POST /admin/reputation-stats` (Phase 5c)
- `AdminAssignJury` → admin backstop
- `AdminCloseCase` → admin backstop

These are already-shipped DTOs intentionally outside the 11. Reference Phase 5b/5c commits if convenient.

### #37 — cargo-test-e2e workflow rust-toolchain override

`.github/workflows/cargo-test-e2e.yml` lines 60-64. The workflow currently uses `dtolnay/rust-toolchain@master` with `toolchain: stable`, silently overriding `rust-toolchain.toml` (pinned to 1.95). Pick **option (A)** — remove the `toolchain: stable` input so the action defers to `rust-toolchain.toml`. The existing comment "Pinned in rust-toolchain.toml; the action reads that file" is correct in spirit; the literal `toolchain: stable` line below it is the bug.

**Out of scope:**
- Do NOT modify any Rust source.
- Do NOT modify any other workflow YAML.
- Do NOT renumber DQ entries or modify any other DQ field.

**Boundaries:**
- Four files edited: `.claude/decision-queue.json` (DQ-6.7 wording), `.claude/PRPs/plans/phase-6-federation.plan.md`, `crates/api/api_common/src/governance.rs` (header comment only — do NOT touch struct definitions), `.github/workflows/cargo-test-e2e.yml`.
- Single commit subject: `docs: DQ-6.7 wording, phase-6 plan MD040 + enum names, admin-DTO carve-out, e2e workflow toolchain (closes #51, #50, #40, #37)`.

## 3. Required reading

- **`.claude/decision-queue.json`** — find DQ-6.7 (entry id 38), read both `question` and `answer`. Decide what to update based on accuracy of each.
- **`.claude/PRPs/plans/phase-6-federation.plan.md`** — Read with `Read` tool offsets `97-127`, `480-493`, `848-877` only. Do NOT load the full file.
- **`migrations/`** — find the enum-creation migrations. `grep -rn "CREATE TYPE\|create type" migrations/` to enumerate; cross-check against the stale names in #50.
- **`crates/api/api_common/src/governance.rs`** — read the header (lines 1-50) and the section around line 287 (the existing carve-out comment).
- **`.github/workflows/cargo-test-e2e.yml`** — full file (likely <100 lines).
- **GitHub issue bodies**: `for n in 51 50 40 37; do gh issue view $n --repo barrie-cork/lemmy --json body --jq .body; echo "---"; done`

## 4. Constraints

**HARD FORBIDS:**
- `cargo *` of any kind. All changes are doc/YAML/JSON.
- Editing any Rust source code (struct definitions, function bodies). Comments inside `governance.rs` are allowed only at the module-header carve-out site.
- Changing `.github/workflows/cargo-validate-*.yml` or any non-target workflow.
- Modifying any DQ entry other than DQ-6.7 / id 38.

**Required behaviour:**
- Single commit subject: `docs: DQ-6.7 wording, phase-6 plan MD040 + enum names, admin-DTO carve-out, e2e workflow toolchain (closes #51, #50, #40, #37)`.
- Trailers: `Closes: barrie-cork/lemmy#51`, `Closes: barrie-cork/lemmy#50`, `Closes: barrie-cork/lemmy#40`, `Closes: barrie-cork/lemmy#37`.
- Push branch and exit.
- DO NOT raise validate-pending DQ. The `governance.rs` edit is a comment-only change (doesn't affect compile); other edits are pure docs/YAML/JSON. No cargo cycle needed.
- DO NOT open a PR.

**File-locality:** four distinct files in different sub-trees — no overlap with any other cluster in this batch. Specifically:
- C8's `governance.rs` edit is comment-only at module header (line ~287). C6c (redaction.rs) and C6a (submit_jury_vote.rs) are different files in the same crate; no commit conflict expected.
- C8's `decision-queue.json` edit only touches DQ-6.7 / id 38. The DQ resolved entries in `pending[]` for issue-96 cluster (DQ #87, #86, #90 if renumbered) are untouched.

**Mid-task DQ push:** if any issue's framing is wrong (e.g. enum names already correct in plan, or DQ-6.7 already updated in trunk), raise `kind: "clarify"` with line excerpts before editing. Do not invent a doc-bug to "fix".
