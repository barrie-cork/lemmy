# Agent D — Phase 6 Layer 3: Outbound publisher

**Model:** opus. **Effort:** maximum — deepest reasoning, most thorough exploration.

**Scope:** Task 74 of `.claude/PRPs/plans/phase-6-federation.plan.md`. Implement `send_local_sanction_notice` and `send_local_trust_attestation` helper functions, re-exported via `crates/apub/apub/src/governance/outbox.rs`.

You are continuing a Brehon governance-fork Phase 6 layered-execution plan. Read `CLAUDE.md`, plan task 74 and pattern `SEND_HELPER`, and the governance-log constants that task 74 adds to `governance_log.rs`. Layer 2 has shipped: AP objects (Agent B, task 72) and AP activities (Agent C, task 73 — receive bodies stubbed pending task 75).

## Worktree

- Advisor has created `../brehon-fork-agent-d-phase6` on branch `agent-d-phase6` cut from `phase-6` tip (post Agents B+C merges).
- **First commands:** `cd ../brehon-fork-agent-d-phase6 && git log --oneline -5 && git submodule update --init --recursive`. Confirm `crates/apub/activities/src/governance/` exists and has stub `publish_sanction_notice.rs` from Agent C.

## Task-hopper envelope

```
scripts/brehon/task-hopper.sh start 74 \
  --agent agent-d --kind cargo_check --layer 3 \
  --label "outbound publisher send_local_sanction_notice" \
  --worktree "$(pwd)"
```

## Task 74 — Outbound publisher

Implement in `crates/apub/activities/src/governance/publish_sanction_notice.rs` and sibling `publish_trust_attestation.rs`:

```rust
pub async fn send_local_sanction_notice(
    case_id: ModerationCaseId,
    context: &Data<LemmyContext>,
) -> LemmyResult<()> { /* steps 1-9 */ }
```

Steps per plan `SEND_HELPER`:

1. Load `ModerationCase` + winning `Sanction` row.
2. Load actor `Person` — for v0 use the **system local admin** per `actor_pseudonym_helper` convention (ADR-010 single-admin v0).
3. Load target entity (`Person`/`Post`/`Comment`/`Community`) to get `target_ap_id`.
4. Call `crate::redaction::scrub(&sanction.reason)` to build the redacted summary (or the relevant scrub fn — audit which module owns it). Reuse — do not re-implement.
5. Construct `SanctionNoticeProtocol` with `published = Utc::now()`, scrubbed `summary`, correct `action`/`scope`/`target`.
6. Wrap in `PublishSanctionNotice { id: generate_activity_id(..., context)?, actor: actor.ap_id.clone().into(), to: [public_url()], cc: [], object: sanction_notice_protocol }`.
7. `let targets = ActivitySendTargets::to_all_instances();`
8. `send_lemmy_activity(context, publish, &actor, targets, /* sensitive = */ false).await?;`
9. `governance_log::append(&mut context.pool(), ENTRY_KIND_FEDERATION_SANCTION_SENT, json!({ "case_id": case_id.0, "sanction_id": sanction.id.0, "target_url": target_ap_id.to_string() }), Some(actor_pseudonym))?;`

Also modify:

- `crates/api/api/src/governance/governance_log.rs` — add four new entry-kind constants per `GOVERNANCE_LOG_APPEND` pattern:
  ```rust
  pub const ENTRY_KIND_FEDERATION_SANCTION_SENT: &str = "federation_sanction_sent";
  pub const ENTRY_KIND_FEDERATION_SANCTION_RECEIVED: &str = "federation_sanction_received";
  pub const ENTRY_KIND_FEDERATION_ATTESTATION_SENT: &str = "federation_attestation_sent";
  pub const ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED: &str = "federation_attestation_received";
  ```
- `crates/apub/apub/src/governance/outbox.rs` — thin re-exports of `send_local_sanction_notice` and `send_local_trust_attestation`. Actual implementations live in the activities crate files; outbox is the public API boundary per [04 §11].
- `crates/apub/apub/src/governance/mod.rs` — `pub mod outbox;` (verify file doesn't already exist; Agent B may have created an empty shell).
- `crates/apub/apub/src/lib.rs` — `pub mod governance;` if not already present.

### Patterns to mirror

- `crates/apub/activities/src/block/block_user.rs:65-96` — `send` helper shape.
- `crates/apub/activities/src/community/report.rs:62-74` — `Report::send` is closest analogy (single-target moderation).

### Gotchas

- **`public_url()`** — the AP magic URL `https://www.w3.org/ns/activitystreams#Public`. Check `activitypub_federation::kinds` for a constant (likely `public()` or similar). Do **not** hardcode the string.
- **Actor signing-key** — local admins always have a valid signing key. If the actor is loaded from `actor_pseudonym_helper`, confirm it returns a `Person` with populated `private_key`.
- **Transaction boundary** — steps 1-7 are DB reads only. Step 8 writes `sent_activity`. Step 9 writes `governance_log`. **Wrap steps 8+9 in `conn.run_transaction()`** per `feedback_multi_write_handlers_need_transactions.md`. If step 9 fails, the activity enqueue must roll back; otherwise we publish an event we can't attribute in the log.
- **`send_local_trust_attestation`** — ship it (builds the activity + enqueues + logs) even though v0 has no handler that calls it. Proves the shape is right. Plan §NOT Building explicitly says this.
- **Do not write `receive_*` functions here** — Agent E owns those in task 75.

### Validate

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub_activities > .claude/build-task74-act.log 2>&1"
tail -20 .claude/build-task74-act.log
echo "act exit: $?"

cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub > .claude/build-task74-apub.log 2>&1"
tail -20 .claude/build-task74-apub.log
echo "apub exit: $?"
```

Both must exit 0.

## Commit

`feat(governance): task 74 — outbound publisher` with plan-template body. Push `agent-d-phase6`.

## Catch-fire triggers

- `send_lemmy_activity` signature drift vs plan's reference at `activities/src/lib.rs:113-143` → DQ entry.
- `redaction::scrub` not reachable from the activities crate (dep-graph issue) → DQ entry; do NOT reimplement scrub.
- Transaction wrap impossible because `send_lemmy_activity` already manages its own tx → DQ entry to resolve ordering; absent a clear answer, DO NOT ship (step 9 must be guaranteed to run iff step 8 commits).
