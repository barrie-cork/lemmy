# Agent F — Phase 6 Layer 4: Wire submit_jury_vote to federation publish

**Model:** opus. **Effort:** maximum — deepest reasoning, most thorough exploration.

**Scope:** Task 76 of `.claude/PRPs/plans/phase-6-federation.plan.md`. Single edit to `crates/api/api/src/governance/submit_jury_vote.rs` — insert a call to `send_local_sanction_notice` when the winning sanction has `scope == SanctionScope::FederatedRecommendation`.

You are continuing a Brehon governance-fork Phase 6 layered-execution plan. Read `CLAUDE.md`, plan task 76, and `.claude/decision-queue.json` DQ-6.5 (branch on scope, not action). Layer 3 has shipped: outbound publisher (Agent D, task 74) + inbound receiver + verify helper (Agent E, tasks 75 + 78).

## Worktree

- Advisor has created `../brehon-fork-agent-f-phase6` on branch `agent-f-phase6` cut from `phase-6` tip.
- **First commands:** `cd ../brehon-fork-agent-f-phase6 && git log --oneline -5 && git submodule update --init --recursive`. Confirm `crates/apub/apub/src/governance/outbox.rs` exports `send_local_sanction_notice`.

## Task-hopper envelope

```
scripts/brehon/task-hopper.sh start 76 \
  --agent agent-f --kind cargo_check --layer 4 \
  --label "wire submit_jury_vote to federation publish" \
  --worktree "$(pwd)"
```

## Task 76 — Wire `submit_jury_vote`

**Surgical edit** to `crates/api/api/src/governance/submit_jury_vote.rs`. Plan §task 76 cites approximate line numbers — they may drift slightly. Use semantic anchoring, not line-number anchoring.

### Where to insert

After the reporter reputation update block (`emit_reputation_event` for the reporter), before the `case_decided` governance_log append. The audit-confirmed sequence:

1. sanction insert (~line 238)
2. sponsor-liability (~line 259)
3. case flip → Decided (~line 273)
4. `public_case_log` append (~line 295)
5. juror reputation deltas (~line 347)
6. reporter reputation delta (~line 385)
7. **[NEW] federation publish — YOU INSERT HERE**
8. `case_decided` log append (~line 397)

### What to insert

```rust
// Phase 6 task 76 — federated recommendation outbound publish.
// Runs inside the post-decision transaction; failure rolls back the
// whole decision so local state never outruns federation.
if winning_sanction.scope == SanctionScope::FederatedRecommendation {
    lemmy_apub::governance::outbox::send_local_sanction_notice(
        case_id,
        &context,
    ).await?;
}
```

(The exact import path depends on how Agent D re-exported. Verify with `rg "pub use .*send_local_sanction_notice" crates/apub/apub/src/governance/`.)

### Invariants

- **Branch on `SanctionScope::FederatedRecommendation`**, NOT on `SanctionAction::FederationQuarantineRecommendation`. (DQ-6.5.) Verify variant name in `crates/db_schema_file/src/enums.rs` first.
- **Inside the existing transaction**. Plan 5b's post-decision block is already `conn.run_transaction()` per `feedback_multi_write_handlers_need_transactions.md`. Confirm the federation call is inside the transaction closure, not outside. If plan 5b did NOT wrap this block in a transaction, you wrap the whole post-decision block (sanction insert → sponsor-liability → case flip → public_log → reputation deltas → federation publish → case_decided) in one `conn.run_transaction()`.
- **Early-exit path** at `submit_jury_vote.rs:192-197` (pre-quorum return) must not touch the federation publish. The scope-check guards this naturally, but double-check: your inserted code is inside the post-decision branch, not before quorum check.
- **Context passing** — `context` is already `Data<LemmyContext>` in the handler scope; pass `&context`.
- **`case_id`** is a local variable at this point; pass it directly.

### Gotchas

- **Regression risk**: existing test `all_mvp_endpoints_return_non_404` (Phase 5c task 68) calls `submit_jury_vote` with `NoAction` / `RemoveContent` votes — these don't trip `FederatedRecommendation`, so no regression. But verify by running the e2e compile target after your commit.
- **`lemmy_apub` dep** — `lemmy_api` may need to add `lemmy_apub` as a dependency if it doesn't already. Check `crates/api/api/Cargo.toml` — if `lemmy_apub` isn't listed, add it under `[dependencies]` with `workspace = true`. Regenerate `Cargo.lock` will happen automatically; stage it with the Cargo.toml edit per `feedback_commit_hygiene_lockfiles_and_task_labels.md`.
- **Do not wrap in a separate transaction** — reuse the existing transaction. Nested transactions in PG don't exist (savepoints only); reusing the outer tx is correct.
- **Do not add matching arms** on `SanctionScope` — the `if` guard avoids exhaustive-match burden. `SanctionScope::Local` and other variants fall through to no-op, correct for v0.

### Validate

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/build-task76.log 2>&1"
tail -20 .claude/build-task76.log
echo "check exit: $?"

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/build-task76-test.log 2>&1"
tail -20 .claude/build-task76-test.log
echo "e2e-compile exit: $?"
```

Both 0. The e2e compile target test ensures existing tests still build; they don't run here.

## Commit

`feat(governance): task 76 — wire submit_jury_vote to federation publish` with plan-template body. Push `agent-f-phase6`.

## Catch-fire triggers

- Plan's line numbers drift by more than ~30 lines from current file → re-anchor semantically, don't stop. Annotate inline: `// NOTE: plan §task 76 cites line ~395; anchored here at line NNN post-phase-5c-rebase`.
- `lemmy_apub` dep addition hits an E0432 cycle → DQ entry; likely means the outbox re-export is misplaced.
- Existing golden-path test breaks unexpectedly → roll back, analyse. The Phase 5c test does not vote `FederatedRecommendation` so it should be neutral.
