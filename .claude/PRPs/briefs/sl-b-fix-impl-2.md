---
role: impl-task
plan_task: 2-fix
phase: v1-SL-b
created: 2026-05-07
related_pr: 119
related_findings: cr-4, cr-5, cr-6, cr-7, cr-22, cr-23
---

# Brief — v1-SL-b fix-impl-2 — PR #119 CR fixes (handler + workflow)

## 1. Role + dispatch line

`[role:impl-task] sl-b-fix-impl-2 — see .claude/PRPs/briefs/sl-b-fix-impl-2.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a **CR-triage fix-in-pr** task addressing 6 CodeRabbit findings on PR #119: 4 substantive on `revoke_endorsement.rs` (cr-4, cr-5, cr-6, cr-7) + 2 on `adr-compliance.yml` (cr-22, cr-23). Scope is **strictly** these 6 findings; no scope creep, no surrounding refactor.

**Companion task:** B2 (advisor-direct) authors a 10th e2e test exercising sponsor-A-then-B grace-window revocation on a separate worker branch (`junior/advisor-sl-b-fix-impl-2-e2e`). Do NOT add e2e tests in this task — e2e edits are PMD #117 hazard for Junior workers and are handled advisor-direct.

## 2. Scope

**Produce:**

1. **One impl commit** with edits to:
   - `crates/api/api_crud/src/governance/revoke_endorsement.rs` (cr-4, cr-5, cr-6, cr-7)
   - `.github/workflows/adr-compliance.yml` (cr-22, cr-23)
2. **Push the worker branch** to `origin/junior/<task-slug>`.
3. **Write a `kind: "validate-pending"` DQ entry** capturing the workflow run id of `cargo-validate-workspace.yml` per Shape G Layer G2. This is a SECOND commit on the same worker branch.

**Do NOT** in this task:

- Touch any file other than the two named + `.claude/decision-queue.json`.
- Add or modify e2e tests in `crates/server/tests/e2e.rs` (advisor handles via B2).
- Refactor surrounding code "while you're in there".
- Modify `.claude/PRPs/plans/**` or `.claude/PRPs/briefs/**`.
- Run cargo locally — Shape G; cargo runs on GH-hosted runners.
- Address `.claude/` ownership-rule complaints (cr-8..cr-20) — those are advisor-rebutted in PR #119 triage; ignore.
- Address machine-local marker / markdownlint findings (cr-1, cr-2, cr-3, cr-24..cr-30) — those are wont-fix.
- Address cr-21 (advisory bypass) — advisor-rebutted in triage as stale (workflow line 137 already gates on `scan_status`).

## 3. Required reading

Read in this order before any Edit:

1. **`.claude/PRPs/reviews/pr-119-comment.md`** — the digest comment posted at https://github.com/barrie-cork/lemmy/pull/119#issuecomment-4396468205 — the canonical statement of the 6 fix-in-pr findings + their bucket rationales. The "fix-in-pr" section is your spec.
2. **`.claude/PRPs/reviews/pr-119-findings.yaml`** — full finding bodies (CR's verbatim wording + AI-agent prompts) for cr-4, cr-5, cr-6, cr-7, cr-22, cr-23. Read each finding's `notes:` field for the cross-reference context (especially cr-6's deferral to v1-SL-c).
3. **`crates/api/api_crud/src/governance/revoke_endorsement.rs`** — full file, especially:
   - Lines 9-39 (architectural-decisions doc-block) — explains rate-limit PRE-TX design + `actor_pseudonym` discipline + the SL-b plan §10 step shapes
   - Lines 70-138 (handler entry + outer wrapper)
   - Lines 143-169 (Step 1 + Step 1.5 idempotent return) — cr-5 fix site
   - Lines 84-106 (PRE-TX rate-limit) — cr-4 fix site
   - Lines 251-270 (Step 6 escape rule + `majority_revocation` arm) — cr-6 fix site
   - Lines 310-333 (Step 8 `endorsement_revoked` log payload) — cr-7 fix site
4. **`.github/workflows/adr-compliance.yml`** — full file, especially:
   - Lines 50-71 (`Determine base and head refs` step) — cr-22 fix site
   - Lines 109-128 (`Comment findings on PR` step) — cr-23 fix site
5. **`.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` §4 + §10** — the watchpoints + step-shape spec the SL-b implementation followed. cr-7 in particular needs you to honour the plan's payload field-name decisions (`sponsor_pseudonym`/`target_pseudonym`, NOT `actor_pseudonym`).
6. **`.claude/lessons/feedback_clippy_test_style.md`** — clippy lint discipline; especially `as_object_mut().insert(...)` pattern.
7. **`.claude/rules/decision-queue.md`** — schema-v2 for the new `validate-pending` entry.
8. **`.claude/runlog/advisor-relays/adhoc-sl-c-baseline-sponsor-count.md`** — the cross-sub-phase coordination note that scopes cr-6 to a NARROW handler-only fix (the authoritative `baseline_sponsor_count` mechanic is deferred to v1-SL-c).

## 3a. Handover from prior cohort

**From PR #119 triage (advisor, 2026-05-07):** SHIPPED but blocked on CR findings.
- PR #119 head: `5683e3dff` on `phase-v1-SL-b`. cargo check + cargo clippy both pass clean baseline (advisor verified locally `2026-05-07T11:50Z`).
- Triage digest posted to PR #119 (issuecomment-4396468205). 6 fix-in-pr / 14 rebut / 0 carry-forward / 10 wont-fix.
- **B2 (advisor-direct)** is authoring the 10th e2e test in parallel on `junior/advisor-sl-b-fix-impl-2-e2e`. Do NOT touch e2e.rs in this task; B2 owns it.
- The pi-session has been writing CI-fix commits on `phase-v1-SL-b` (last several auto(pi)/chore(pi)). Your branch should base off `5683e3dff` (current phase tip); the daemon finalize-merge will rebase.

**Phase tip:** `5683e3dff chore(skills): add cross-harness session-retro skill`.

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branched off `phase-v1-SL-b` (tip `5683e3dff`).
- `git branch --show-current` should return a `junior/role-impl-task-sl-b-fix-impl-2-...` branch (adapt to your actual worktree branch name).
- Two commits at task end:
  1. Impl: `fix(v1-SL-b): PR #119 CR fixes — handler authz + payload schema + workflow polish`
  2. DQ: `chore(decision-queue): impl raised DQ #<next-id> — sl-b-fix-impl-2 validate-pending`

### Implementation discipline — the SIX edits

**cr-5 (critical) — Move authz check ahead of idempotent early-return**

Anchor on lines 153-174 in `revoke_endorsement.rs`. Current order:

```rust
  // Step 1: load endorsement FOR UPDATE; idempotency check inside tx
  let row: Endorsement = endorsement::table
    .filter(endorsement::id.eq(data.endorsement_id))
    .for_update()
    .select(Endorsement::as_select())
    .first(conn)
    .await?;

  // Step 1.5: idempotency (PRD §5.4) — re-revocation is a no-op.
  if let Some(existing_revoked_at) = row.revoked_at {
    return Ok(RevokeEndorsementResponse {
      endorsement_id: row.id,
      revoked_at: existing_revoked_at,
      liability_chain_severed_for_cases: vec![],
    });
  }

  // Step 2: capability check — self-revoke or admin.
  if !is_admin_caller && row.from_person_id != caller_id {
    return Err(LemmyErrorType::NotFound.into());
  }
```

**Required new order:** Step 1 (load row), then Step 2 (capability check), THEN Step 1.5 (idempotent return). Keep all existing variable names + comments. Update step labels to reflect new order:

```rust
  // Step 1: load endorsement FOR UPDATE.
  let row: Endorsement = endorsement::table
    .filter(endorsement::id.eq(data.endorsement_id))
    .for_update()
    .select(Endorsement::as_select())
    .first(conn)
    .await?;

  // Step 2: capability check — self-revoke or admin (cr-5 PR #119:
  // before the idempotent early-return; otherwise unauthorised callers
  // distinguish revoked-vs-not-found via revoked_at leak).
  if !is_admin_caller && row.from_person_id != caller_id {
    return Err(LemmyErrorType::NotFound.into());
  }

  // Step 3: idempotency (PRD §5.4) — re-revocation is a no-op for
  // authorised callers (was Step 1.5 pre-cr-5).
  if let Some(existing_revoked_at) = row.revoked_at {
    return Ok(RevokeEndorsementResponse {
      endorsement_id: row.id,
      revoked_at: existing_revoked_at,
      liability_chain_severed_for_cases: vec![],
    });
  }
```

Re-number subsequent step comments downstream of this edit so they remain
sequential:
- Old "Step 2: capability check" → already moved (becomes new Step 2).
- Old "Step 3: UPDATE endorsement.revoked_at" → renumber to "Step 4".
- Old "Step 4: UPDATE matching surety row" → renumber to "Step 5".
- Old "Step 5: re-query active sponsors" → renumber to "Step 6".
- Old "Step 6: apply escape rule" → renumber to "Step 7".
- Old "Step 7: recompute snapshots" → renumber to "Step 8".
- Old "Step 8: emit endorsement_revoked log entry" → renumber to "Step 9".

**cr-4 (major) — Defer rate-limit to after idempotent read**

The plan §10 + handler doc-comment specify rate-limit runs PRE-TX. CR's recommendation is that retried-already-revoked requests should NOT trip the rate-limit. The cleanest fix preserving the PRE-TX design: add a quick PRE-TX idempotency probe BEFORE the rate-limit count.

Anchor on lines 84-106 in `revoke_endorsement.rs`. Currently `recent_count` (lines 95-100) executes BEFORE checking if the endorsement is already revoked. Insert a probe immediately before:

```rust
  // PRE-TX: idempotency probe (cr-4 PR #119) — already-revoked retries
  // skip rate-limit accounting. Cheap single-row SELECT; the tx-level
  // FOR UPDATE re-check at Step 3 is the authoritative idempotency
  // gate. This probe avoids 429s on retries of a successful revoke.
  let already_revoked: Option<DateTime<Utc>> = endorsement::table
    .filter(endorsement::id.eq(data.endorsement_id))
    .select(endorsement::revoked_at)
    .first::<Option<DateTime<Utc>>>(conn)
    .await
    .optional()?
    .flatten();
  // If already revoked, fall through to the tx (it'll re-load FOR UPDATE
  // and idempotent-return); skip the rate-limit count + bypass logic.
  let (rate_limit_per_day, recent_count, bypass_recorded);
  if already_revoked.is_some() {
    rate_limit_per_day = i64::MAX;
    recent_count = 0;
    bypass_recorded = false;
  } else {
    // PRE-TX: rate-limit count (admin bypasses — Watch 14 / R1 i64 discipline).
    rate_limit_per_day = config::get_int(
      &mut ConfigCache::new(),
      &mut context.pool(),
      Scope::Instance,
      "liability.revoke_rate_limit_per_day",
    )
    .await?;
    let cutoff: DateTime<Utc> = Utc::now() - Duration::hours(24);
    recent_count = endorsement::table
      .filter(endorsement::from_person_id.eq(caller_id))
      .filter(endorsement::revoked_at.gt(cutoff))
      .select(count_star())
      .get_result(conn)
      .await?;
    bypass_recorded = is_admin_caller && recent_count >= rate_limit_per_day;
    if !is_admin_caller && recent_count >= rate_limit_per_day {
      return Err(LemmyErrorType::TooManyRequests.into());
    }
  }
```

You'll need:
- `use diesel::OptionalExtension;` (already imported via `diesel::ExpressionMethods` block; verify with `grep -n "OptionalExtension" crates/api/api_crud/src/governance/revoke_endorsement.rs` — add to the import list if missing).
- The `data.endorsement_id` reference is preserved from `Json<RevokeEndorsement>`.

**Why this shape:** PRE-TX probe is a cheap `SELECT revoked_at FROM endorsement WHERE id = ?`. The transaction's Step 1 still does `FOR UPDATE` re-check (race-free authoritative gate). Skipping rate-limit on retries doesn't bypass the tx's idempotency — both probes converge to the same answer.

**cr-6 (major) — Narrow handler fix; defer to v1-SL-c**

Anchor on lines 251-270. Current `majority_revocation` arm:

```rust
  let escape_rule = config::get_text(
    &mut config,
    &mut (&mut *conn).into(),
    scope,
    "liability.multi_sponsor_escape_rule",
  )
  .await?;

  // Step 5: re-query active sponsors for `to_person_id` (post-step-4 state).
  let active_sponsor_count: i64 = surety::table
    .filter(surety::sponsored_id.eq(row.to_person_id))
    .filter(surety::revoked_at.is_null())
    .select(count_star())
    .get_result(conn)
    .await?;

  // Step 6: apply escape rule.
  let escapes = match escape_rule.as_str() {
    "all_revocation" => active_sponsor_count == 0,
    "majority_revocation" => {
      // pre-revoke sponsor count = active + 1 (this caller just revoked)
      let pre_count = active_sponsor_count + 1;
      let revoked_since_decision = pre_count - active_sponsor_count;
      revoked_since_decision * 2 > pre_count
    }
    // "any_revocation" (default) + unknown/NULL fallback (defensive — no error)
    _ => true,
  };
```

**Required edit:** Replace the `"majority_revocation"` arm so it falls through to `any_revocation` (default true) with a TODO citing the SL-c deferral. Keep the comment block explaining the deferral. New shape:

```rust
  // Step 7: apply escape rule.
  let escapes = match escape_rule.as_str() {
    "all_revocation" => active_sponsor_count == 0,
    // cr-6 PR #119: the synchronous handler cannot correctly evaluate
    // majority_revocation without a persisted baseline_sponsor_count
    // captured at the Decided -> SponsorLiabilityPending transition.
    // The authoritative evaluator lives in v1-SL-c's grace-check
    // (sponsor_liability_grace.rs::evaluate_escape_conditions). Until
    // that lands, fall through to any_revocation behaviour for cases
    // where the operator selected "majority_revocation". See
    // .claude/runlog/advisor-relays/adhoc-sl-c-baseline-sponsor-count.md.
    // TODO(v1-SL-c): restore majority threshold once
    // moderation_case.baseline_sponsor_count column lands.
    "majority_revocation" => true,
    // "any_revocation" (default) + unknown/NULL fallback (defensive — no error)
    _ => true,
  };
```

(Note: the surrounding step comment was renumbered from "Step 6" to "Step 7" by the cr-5 renumber cascade; verify the actual number you write matches the cr-5 cascade.)

**cr-7 (major) — endorsement_revoked payload schema**

Anchor on lines 310-333. Current `endorsement_revoked` payload (line 315-321):

```rust
  let mut payload = json!({
    "sponsor_pseudonym": caller_pseudonym,
    "target_pseudonym": target_pseudonym,
    "community_id": row.community_id.map(|c| c.0),
    "reason": data.reason,
    "liability_chain_severed_for_cases": severed.iter().map(|c| c.0).collect::<Vec<_>>(),
  });
```

**Required edit:** Add `version: 1` and `endorsement_id` per CR's recommendation. **DO NOT** rename `sponsor_pseudonym` / `target_pseudonym` to `actor_pseudonym` — those are plan-§10-mandated semantic field names per DQ #140 and richer than a single `actor_pseudonym`. CR's recommendation conflates `liability_escape_reason` schema (single `actor_pseudonym`) with `endorsement_revoked` schema (sponsor + target distinguishable). New shape:

```rust
  let mut payload = json!({
    "version": 1,
    "endorsement_id": row.id.0,
    "sponsor_pseudonym": caller_pseudonym,
    "target_pseudonym": target_pseudonym,
    "community_id": row.community_id.map(|c| c.0),
    "reason": data.reason,
    "liability_chain_severed_for_cases": severed.iter().map(|c| c.0).collect::<Vec<_>>(),
  });
```

The `if bypass_recorded { obj.insert(...) }` block beneath is unchanged (already-shipped clippy fix from sl-b-fix-impl-1 stays).

**cr-22 (major) — github.sha instead of git rev-parse HEAD**

Anchor on lines 57-60 of `.github/workflows/adr-compliance.yml`. Current:

```yaml
          elif [ "${{ github.event_name }}" = "workflow_dispatch" ]; then
            BASE_REF="${{ github.event.inputs.base_ref }}"
            HEAD_SHA="${INPUTS_HEAD_SHA:-$(git rev-parse HEAD)}"
            PR_NUMBER=""
```

**Replace with:**

```yaml
          elif [ "${{ github.event_name }}" = "workflow_dispatch" ]; then
            BASE_REF="${{ github.event.inputs.base_ref }}"
            HEAD_SHA="${INPUTS_HEAD_SHA:-${{ github.sha }}}"
            PR_NUMBER=""
```

**cr-23 (nit) — refs.outputs.pr_number consistency**

Anchor on lines 109-128 of `.github/workflows/adr-compliance.yml`. The `Comment findings on PR` step. Current line 126:

```yaml
              issue_number: context.payload.pull_request.number,
```

**Replace with:**

```yaml
              issue_number: parseInt('${{ steps.refs.outputs.pr_number }}', 10),
```

(This pulls the PR number from the same source as the step's `if:` gate at line 110, eliminating the dual-source-of-truth.)

### Verification (no cargo locally)

Per Shape G, you do NOT run cargo. Verification is done by the workspace-check workflow on the GH-hosted runner triggered by your push.

Quick visual checks on the worker branch BEFORE the second commit:

```bash
# All 6 sites edited
git diff HEAD -- crates/api/api_crud/src/governance/revoke_endorsement.rs | head -150
git diff HEAD -- .github/workflows/adr-compliance.yml | head -50

# No stray edits to e2e.rs or other files
git diff --stat HEAD
# Expected: exactly 2 files changed (revoke_endorsement.rs + adr-compliance.yml)
```

If `git diff --stat` shows any file other than the two expected, STOP and revert before push.

### Validate-pending DQ entry shape (Shape G Layer G2)

```json
{
  "id": <next-int>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<NOW_ISO>",
  "subject": "sl-b-fix-impl-2 workspace-check validate-pending — PR #119 CR fixes",
  "branch": "junior/<task-slug>",
  "phase_task": "sl-b-fix-impl-2",
  "workflow_run_id": <int from gh run list>,
  "head_sha": "<short-sha of impl commit>",
  "commands": [
    "cargo check --workspace --features full",
    "cargo clippy --workspace --features full --no-deps -- -D warnings",
    "cargo test --no-run -p lemmy_server --test e2e --features full"
  ],
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "context": "PR #119 CR triage fix: cr-4 (rate-limit ordering), cr-5 (authz before idempotent return — CRITICAL), cr-6 (majority_revocation deferred to v1-SL-c per adhoc-sl-c-baseline-sponsor-count.md), cr-7 (endorsement_revoked payload version+endorsement_id), cr-22 (github.sha vs git rev-parse), cr-23 (refs.outputs.pr_number consistency). Companion B2 (10th e2e test) on separate junior/advisor-sl-b-fix-impl-2-e2e branch."
}
```

After pushing the impl commit, capture the `workflow_run_id`:

```bash
gh run list --repo barrie-cork/lemmy --branch <your-worker-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId --jq '.[0].databaseId'
```

If `gh run list` returns empty, wait up to 60 seconds and retry.

### Hard refusals

- Do NOT modify any file other than the three named (handler + workflow + DQ).
- Do NOT touch `crates/server/tests/e2e.rs` — B2 (advisor-direct) handles e2e additions.
- Do NOT add `unwrap()` or `expect()` (`feedback_clippy_test_style.md`).
- Do NOT rename `sponsor_pseudonym`/`target_pseudonym` to `actor_pseudonym` — plan §10 + DQ #140 mandate the semantic field names.
- Do NOT remove the `majority_revocation` arm; only re-route its body to `true` with the TODO + relay-citation.
- Do NOT push the worker branch BEFORE both commits land.
- Do NOT raise more than one DQ entry; the single `validate-pending` covers all 6 fixes.

## 5. Acceptance

Fix-impl-2 passes if:

- `revoke_endorsement.rs` step ordering: capability check (Step 2) appears BEFORE idempotent return (Step 3); steps thereafter renumbered Step 4..9.
- `revoke_endorsement.rs` PRE-TX block has `already_revoked` probe + branch on rate-limit accounting.
- `revoke_endorsement.rs` `"majority_revocation"` match arm body returns `true` with TODO comment citing relay.
- `revoke_endorsement.rs` `endorsement_revoked` payload contains `"version": 1` + `"endorsement_id": row.id.0`.
- `adr-compliance.yml:60` uses `${{ github.sha }}`.
- `adr-compliance.yml:126` uses `parseInt('${{ steps.refs.outputs.pr_number }}', 10)`.
- `git diff --stat HEAD~1` shows exactly 2 files: `revoke_endorsement.rs` + `adr-compliance.yml`.
- One impl commit + one DQ-entry commit on the worker branch.
- DQ entry written with `kind: "validate-pending"`, `from: "impl"`, populated `workflow_run_id`.
- ci-watcher's later poll returns `conclusion: "success"`.
