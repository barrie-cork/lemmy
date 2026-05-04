---
role: impl-task
plan_task: 2
phase: v1-SL-b
created: 2026-05-04
related_dq: null
---

# Brief — v1-SL-b Task 2 — Create revoke_endorsement.rs handler + module wiring

## 1. Role + dispatch line

`[role:impl-task] sl-b-impl-2 — see .claude/PRPs/briefs/sl-b-impl-2.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 2 from
`.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` §13 — create the
`revoke_endorsement.rs` handler module + wire it via `mod.rs`. **Two file
modifications, one impl commit + one DQ entry commit.**

## 2. Scope

**Produce:**

1. **One impl commit** with:
   - **NEW file:** `crates/api/api_crud/src/governance/revoke_endorsement.rs`
     containing the full handler module per plan §10.1 + §10.2 + §10.4 +
     §10.5 + §10.7 (~250 lines including doc-comments, use block, outer
     handler, `process_revocation` body, payload assembly).
   - **MODIFIED file:** `crates/api/api_crud/src/governance/mod.rs` —
     add `pub mod revoke_endorsement;` alphabetically (between
     `create_report` and `request_appeal`).
2. **Push the worker branch** to `origin/junior/<task-slug>`.
3. **Write a `kind: "validate-pending"` DQ entry** capturing the
   workflow run id of `cargo-validate-workspace.yml` per Shape G Layer
   G2. This is a SECOND commit on the same worker branch.

**Do NOT** in this task:

- Modify `crates/api/api_common/src/governance.rs` (Task 1 — already
  shipped).
- Modify `crates/api/routes/src/lib.rs` (Task 3).
- Touch `crates/server/tests/e2e.rs` (Tasks 4-12).
- Run cargo locally — Shape G; cargo runs on GH-hosted runners.
- Modify any e2e tests, helpers, or fixtures.
- Modify `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.
- Add new ENTRY_KIND consts or CaseStatus variants — substrate already
  in place from SL-a.

## 3. Required reading

Read in this order before writing the new file:

1. **Plan §13 Task 2** (`.claude/PRPs/plans/v1-sponsor-liability-b.plan.md`)
   — the FILES YAML + IMPLEMENT block + GOTCHAs.
2. **Plan §10.1** (outer handler shape, lines 1002-1078) — verbatim
   handler skeleton.
3. **Plan §10.2** (`process_revocation` transaction body, lines
   1079-1257) — verbatim 8-step inside-transaction body.
4. **Plan §10.4** (governance_log payload schema, lines 1310-1354) —
   exact JSON shapes for `endorsement_revoked` + `sponsor_liability_escaped`
   payloads, including `version: 1` + `actor_pseudonym` + the
   `rate_limit_bypassed: true` field for admin bypass case.
5. **Plan §10.5** (reason validation, lines 1355-1370) — empty-after-trim
   rejection via `LemmyErrorType::Unknown("revoke-endorsement reason
   required".to_string())`.
6. **Plan §10.7** (rate-limit + bypass, lines 1399 onward) — pre-tx
   count + admin-bypass annotation logic.
7. **Plan §4.1 + §4.2** (architectural decisions + 14 watchpoints) —
   confirms the inside-tx step ordering, ConfigCache per-case discipline
   per DQ #142, ADR-015 actor_pseudonym discipline, ADR-013 no new match
   sites.
8. **`crates/api/api_crud/src/governance/create_endorsement.rs`** — read
   the full file (361 lines). Canonical mirror. Pay attention to:
   - Module doc-comment shape (lines 1-34).
   - Use block (lines 35-69) — the brief lists the exact use block to
     write but the mirror is the source of truth on patterns.
   - Cooldown count pattern (lines 195-214) — read-then-error pattern;
     SL-b's rate-limit pre-tx mirrors this shape.
   - actor_pseudonym_helper::get_or_create call site (lines 126-127).
   - Inside-tx writes via `run_transaction(...)` (lines 230-330+).
   - `recompute_snapshot` calls inside tx (lines 307-309) — SL-b calls
     it twice (sponsor + sponsee).
   - `governance_log::append` call shape (lines 311-328).
9. **`crates/api/api/src/governance/admin_close_case.rs`** lines 23-105
   — admin gate pattern + reason validation idiom + governance_log
   append on admin write.
10. **`crates/api/api/src/governance/config.rs`** lines 209-260
    (ConfigCache shape) and line 937
    (`DEFAULT_LIABILITY_MULTI_SPONSOR_ESCAPE_RULE`).
11. **`.claude/rules/decision-queue.md`** — schema-v2 for the
    `validate-pending` entry shape.
12. **Lessons** (Glob `.claude/lessons/`):
    - `feedback_multi_write_handlers_need_transactions.md` —
      load-bearing; rate-limit pre-tx, idempotency check INSIDE tx,
      every UPDATE inside one closure.
    - `feedback_features_full_workspace_only.md` — workspace-check
      activates ts-rs derives; SL-b only uses common's ts-rs (already
      shipped Task 1).
    - `feedback_clippy_test_style.md` — R1: `i64` consistency for
      `count_star()` + `get_int` results.
    - `feedback_clippy_rerun_after_fix.md` — re-run clippy locally if
      a refactor unmasks lints.
    - `feedback_pr_per_phase.md` — one impl commit + separate DQ
      commit; do not bundle.

## 3a. Handover from prior cohort

**From Task 1 (sl-b-impl-1, task #116):** SHIPPED.
- `crates/api/api_common/src/governance.rs` now exports:
  - `RevokeEndorsement { endorsement_id: EndorsementId, reason: String }` —
    derives `Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash`
    (NO `Copy`).
  - `RevokeEndorsementResponse { endorsement_id, revoked_at: DateTime<Utc>,
    liability_chain_severed_for_cases: Vec<ModerationCaseId> }`.
- DQ #144 resolved: workspace-check passed (run 25331647761,
  `result: pass`).
- Phase tip: `db2630d53 chore(ci-watcher): merge sl-b-ci-watcher-1 — DQ
  #144 mutated validate-pending pass`.
- Workspace compiles cleanly with the new DTOs.

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branched off `phase-v1-SL-b` (tip
  `db2630d53`).
- `git branch --show-current` should return a `junior/role-impl-task-...`
  branch (adapt to your actual worktree branch name).
- Two commits at task end:
  1. Impl: `feat(v1-SL-b): create revoke_endorsement handler + module wiring (task 2)`
  2. DQ: `chore(decision-queue): impl raised DQ #<next-id> — sl-b-impl-2 validate-pending`
- After both commits land, push the worker branch to origin. Junior's
  daemon finalize-merges into `phase-v1-SL-b` on completion.

### Implementation discipline — handler file

- **Write the file from scratch** mirroring `create_endorsement.rs`
  shape. Do not duplicate `create_endorsement` logic — `revoke_endorsement`
  is structurally similar but semantically distinct (UPDATE not INSERT;
  multiple downstream UPDATEs; severance loop).
- **8-step inside-tx body** per plan §4.1 + §10.2:
  1. Load endorsement with `FOR UPDATE`. Verify `revoked_at IS NULL`.
     If already revoked, EARLY-RETURN existing `revoked_at` with empty
     severed-cases vec (idempotency).
  2. Verify capability — `endorsement.from_person_id == caller_id` OR
     `is_admin_caller`. Else `Err(LemmyErrorType::NotFound.into())`.
  3. UPDATE `endorsement SET revoked_at = now()`.
  4. UPDATE the matching `surety` row by triple
     `(from_person_id, to_person_id, community_id)`.
  5. Re-query active sponsors for `to_person_id`.
  6. Grace-window evaluation loop — query `moderation_case` rows with
     `status = 'SponsorLiabilityPending' AND target_person_id =
     endorsement.to_person_id AND grace_expires_at > now()`. For each
     case, read community-scoped escape rule via
     `config::get_text(cache, conn, Scope::Community(case.community_id),
     "liability.multi_sponsor_escape_rule")` (per DQ #142 — per-case,
     not pre-loop). Apply rule (`any_revocation` default,
     `all_revocation`, `majority_revocation`, fall back to
     `any_revocation` on unknown/NULL). On escape met: UPDATE case
     status + log entry.
  7. Recompute snapshots — sponsor (`caller_id`) AND sponsee
     (`endorsement.to_person_id`).
  8. Emit `endorsement_revoked` log entry ALWAYS. Payload includes
     `rate_limit_bypassed: true` ONLY when (admin AND
     would-have-been-rate-limited).

### Implementation discipline — mod.rs

- **Single-line addition.** Add `pub mod revoke_endorsement;`
  alphabetically between `pub mod create_report;` and
  `pub mod request_appeal;`. Do not reorder anything else.

### GOTCHAs (cite directly to inform Edits)

1. **TOCTOU on re-revoke:** the load + revoked_at-NULL check
   MUST be inside the `run_transaction` closure with `FOR UPDATE`.
   Separate-read-then-tx-write is a TOCTOU bug. Per plan §4.2 #1.
2. **i64 typing (R1):** `recent_count: i64` from `count_star()`;
   `rate_limit_per_day: i64` from `get_int`. Direct `>=` compare.
   No `as` cast. Per plan §13 GOTCHA + `feedback_clippy_test_style.md`.
3. **Scope::Community on Option<CommunityId>:** `case.community_id` is
   `Option<CommunityId>`. Match-pick the right scope (e.g.
   `Scope::Community(c)` if `Some(c)`, else `Scope::Instance`).
   Per plan §13 GOTCHA.
4. **No new ADR-013 match site:** the handler uses Diesel filter
   on `case.status`, NOT a Rust `match case.status { ... }`. SL-b
   adds zero match sites. Per plan §4.2 #4 + §13 GOTCHA.
5. **`actor_pseudonym_helper::get_or_create` BEFORE `run_transaction`:**
   pseudonym lookup is read-only; mirror
   `create_endorsement.rs:126-127`. Per plan §4.1 + ADR-015.
6. **`liability_escape_reason` JSONB schema:** mandatory keys
   `version: 1`, `reason: "sponsor_revoked"`, `actor_pseudonym`,
   `endorsement_id`. NEVER raw `caller_id` — that's a GDPR-013
   violation, catch-fire. Per ADR-015 + OQ-V1-SL-05 + plan §4.2 #4.
7. **`run_transaction` closure scoping:** `data_for_tx` and
   `pseudonym_for_tx` are `move`'d into the closure;
   `is_admin_caller`, `bypass_recorded`, `caller_id` are `Copy` and
   cross by value. Per plan §13 GOTCHA.
8. **Two `recompute_snapshot` calls:** sponsor (`caller_id`) AND
   sponsee (`endorsement.to_person_id`). Inside the same tx. Mirror
   `create_endorsement.rs:307-309`. Per plan §4.2 #6.
9. **`endorsement_revoked` log entry ALWAYS:** even if no severance
   (i.e. liability_chain_severed_for_cases is empty Vec). Per plan
   §4.1 + §10.4. The severance log entry (`sponsor_liability_escaped`)
   is per-case; the `endorsement_revoked` entry is per-handler-call.

### Validate-pending DQ entry shape (Shape G Layer G2)

Per `.claude/rules/decision-queue.md`:

```json
{
  "id": <next-int>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<NOW_ISO>",
  "subject": "sl-b-impl-2 workspace-check validate-pending",
  "branch": "junior/<task-slug>",
  "phase_task": "sl-b-impl-2",
  "workflow_run_id": <int from gh run list>,
  "commands": ["cargo check --workspace --features full", "cargo clippy --workspace --features full --no-deps -- -D warnings", "cargo test --no-run -p lemmy_server --test e2e"],
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

After pushing the impl commit, capture the `workflow_run_id`:

```bash
gh run list --repo barrie-cork/lemmy --branch <your-worker-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId --jq '.[0].databaseId'
```

If `gh run list` returns empty, wait up to 60 seconds and retry.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.
- Any pending DQ entry written from this task uses `from: "impl"`.

### Hard refusals

- Do NOT modify any file outside the FILES YAML block (handler file +
  mod.rs + decision-queue.json).
- Do NOT add a `match case.status` arm anywhere — SL-b adds zero ADR-013
  match sites. Use `.filter()` on Diesel queries.
- Do NOT inline raw `caller_id` in `liability_escape_reason` JSONB —
  use pseudonym always.
- Do NOT skip the FOR UPDATE lock at step 1.
- Do NOT pre-compute the escape rule once before the loop — read per-case
  inside the loop (ConfigCache deduplicates) per DQ #142.
- Do NOT bundle the impl commit and DQ entry commit. Two commits.
- Do NOT push the worker branch BEFORE both commits land.
- Do NOT add new ENTRY_KIND consts. Use existing
  `ENTRY_KIND_ENDORSEMENT_REVOKED` (line 195) and
  `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` (line 197) imported from
  `lemmy_api::governance::governance_log`.
- Do NOT touch `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.
- Do NOT enforce step-up auth — PRD §12.3 reserves it for v2.

## 5. Acceptance

Task 2 passes if:

- `crates/api/api_crud/src/governance/revoke_endorsement.rs` exists with:
  - `pub async fn revoke_endorsement(` outer handler.
  - `async fn process_revocation(` inside-tx body fn.
  - Imports for `ENTRY_KIND_ENDORSEMENT_REVOKED` AND
    `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` AND
    `actor_pseudonym_helper::get_or_create` AND `recompute_snapshot`
    AND `governance_log::append`.
- `crates/api/api_crud/src/governance/mod.rs` contains
  `pub mod revoke_endorsement;` line, alphabetically positioned.
- One impl commit + one DQ-entry commit on the worker branch.
- Worker branch pushed to origin.
- DQ entry written with `kind: "validate-pending"`, `from: "impl"`,
  populated `workflow_run_id`.
- ci-watcher's later poll returns `conclusion: "success"`.
