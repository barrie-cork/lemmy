# Agent E — Phase 6 Layer 3: Inbound receiver + verify helper

**Model:** opus. **Effort:** maximum — deepest reasoning, most thorough exploration.

**Scope:** Tasks 75 + 78 of `.claude/PRPs/plans/phase-6-federation.plan.md`. Free functions `receive_remote_sanction_notice` and `receive_remote_trust_attestation` in `crates/apub/apub/src/governance/inbox.rs`, plus a thin `verify_governance_activity_domain` wrapper in `crates/apub/apub/src/governance/verify.rs`. Also wires the Activity::receive bodies that Agent C stubbed in task 73.

You are continuing a Brehon governance-fork Phase 6 layered-execution plan. Read `CLAUDE.md`, plan tasks 75 + 78, the stub bodies Agent C wrote in task 73, and `.claude/decision-queue.json` DQ-6.2 (signature = `activity.id`) + DQ-6.3 (rely on `ReceivedActivity` dedup).

## Worktree

- Advisor has created `../brehon-fork-agent-e-phase6` on branch `agent-e-phase6` cut from `phase-6` tip (post Agents B+C merges; can run in parallel with Agent D per the original plan but we are running sequentially, so this is post-Agent-D).
- **First commands:** `cd ../brehon-fork-agent-e-phase6 && git log --oneline -5 && git submodule update --init --recursive`. Confirm `crates/apub/apub/src/governance/outbox.rs` exists (Agent D). The `governance/inbox.rs` and `governance/verify.rs` files do NOT exist yet — you create them.

## Task-hopper envelope (two tasks, serial)

```
scripts/brehon/task-hopper.sh start 75 \
  --agent agent-e --kind cargo_check --layer 3 \
  --label "inbound receiver receive_remote_sanction_notice" \
  --worktree "$(pwd)"
# implement task 75, commit
scripts/brehon/task-hopper.sh complete 75 --commit-sha "$(git rev-parse --short HEAD)"

scripts/brehon/task-hopper.sh start 78 \
  --agent agent-e --kind cargo_check --layer 3 \
  --label "verify.rs signature helper" \
  --worktree "$(pwd)"
# implement task 78, commit
scripts/brehon/task-hopper.sh complete 78 --commit-sha "$(git rev-parse --short HEAD)"
```

## Task 75 — Inbound receiver

Create `crates/apub/apub/src/governance/inbox.rs` with two free functions:

```rust
pub async fn receive_remote_sanction_notice(
    activity: PublishSanctionNotice,
    context: &Data<LemmyContext>,
) -> LemmyResult<()>

pub async fn receive_remote_trust_attestation(
    activity: PublishTrustAttestation,
    context: &Data<LemmyContext>,
) -> LemmyResult<()>
```

Steps for `receive_remote_sanction_notice` per plan §task 75:

1. Signature **already verified** by `activitypub_federation::actix_web::inbox::receive_activity_with_hook` at the HTTP layer. Do not re-verify.
2. Schema validation — `activity.object.action` / `.scope` deserialising successfully already proves known variants (Diesel derive rejects unknown). Trust this.
3. Build `RemoteSanctionNoticeInsertForm`:
   ```rust
   RemoteSanctionNoticeInsertForm {
       source_instance: activity.actor.inner().domain().unwrap_or("").to_string(),
       target_url:      activity.object.target.to_string(),
       action:          activity.object.action,
       scope:           activity.object.scope,
       summary:         activity.object.summary.clone(),
       published_at:    activity.object.published,
       signature:       activity.id.to_string(),   // DQ-6.2 resolved
       local_case_id:   None,                       // DQ-6.3 resolved — ADR-006 no auto-apply
   }
   ```
4. `RemoteSanctionNotice::create(&mut context.pool(), &form).await?;`
5. `governance_log::append(&mut context.pool(), ENTRY_KIND_FEDERATION_SANCTION_RECEIVED, json!({ "source_instance": form.source_instance, "target_url": form.target_url, "action": form.action.to_string() }), None)?;` — `actor_pseudonym = None` because the actor is remote.
6. **NEVER** call into local sanction application. No `Sanction::create`, no `emergency_remove_open_case`, nothing that enforces the inbound notice locally. ADR-006.

`receive_remote_trust_attestation` is the same shape, writes to `FederationAttestation` table with `actor_url = activity.actor.inner().to_string()` and log kind `ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED`.

### Wire the Activity::receive bodies Agent C stubbed

Update `crates/apub/activities/src/governance/publish_sanction_notice.rs`:

```rust
async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    lemmy_apub::governance::inbox::receive_remote_sanction_notice(self, context).await
}
```

Same for `publish_trust_attestation.rs`. `publish_label.rs` stays as `Ok(())`.

### Modifications

- `crates/apub/apub/src/governance/mod.rs` — add `pub mod inbox;`
- `crates/apub/apub/src/governance/inbox.rs` — create
- `crates/apub/activities/src/governance/publish_sanction_notice.rs` — replace stub receive body with wire call
- `crates/apub/activities/src/governance/publish_trust_attestation.rs` — same

### Gotchas

- **`.domain()` on `Url`** returns `Option<&str>` — the unwrap_or above is a fallback; if the actor URL lacks a domain, log a warn but don't error (v0 lenient).
- **`source_instance` vs `actor_url`** — the design doc's `remote_sanction_notice.source_instance` is the hostname, not the full URL. Use `.domain()`, not the full URL string.
- **Do not add a unique index / on_conflict().do_nothing()** on the insert — DQ-6.3 resolved. Idempotency lives at the `ReceivedActivity` layer.
- **Plugin-hook stability** — you are not touching PM path hooks (private-message). `.claude/rules/pm-plugin-hooks-stable.md` does not apply. Governance hooks are additive; you're free to add federation hooks later without this constraint.

### Commit task 75

`feat(governance): task 75 — inbound receiver` per plan §task 75 commit template.

### Validate task 75

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub > .claude/build-task75.log 2>&1"
tail -20 .claude/build-task75.log
echo "apub exit: $?"

cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub_activities > .claude/build-task75-act.log 2>&1"
tail -20 .claude/build-task75-act.log
echo "act exit: $?"
```

Both 0.

## Task 78 — verify.rs signature helper

Create `crates/apub/apub/src/governance/verify.rs`:

```rust
use activitypub_federation::protocol::verification::verify_domains_match;
use lemmy_utils::error::LemmyResult;
use url::Url;

/// Shared verification helpers for governance federation activities.
/// Thin wrappers today; future custom verification (e.g. jury-panel
/// attestation strength) will be added here per [04 §11].
pub fn verify_governance_activity_domain(
    activity_id: &Url,
    expected_domain: &Url,
) -> LemmyResult<()> {
    verify_domains_match(activity_id, expected_domain)
}
```

Update `crates/apub/apub/src/governance/mod.rs` with `pub mod verify;`.

### Validate task 78

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub > .claude/build-task78.log 2>&1"
tail -20 .claude/build-task78.log
echo "exit: $?"
```

Exit 0.

### Commit task 78

`feat(governance): task 78 — verify.rs signature helper`. Push `agent-e-phase6` after both commits.

## Catch-fire triggers

- Cross-crate visibility error when wiring `lemmy_apub::governance::inbox::receive_remote_sanction_notice` from the activities crate → DQ entry (likely a missing `pub` somewhere or a feature-flag mismatch).
- `activity.actor.inner().domain()` returns `None` → that's a latent-v0 edge case but not a blocker; log a warn and use empty string.
- Any temptation to add a `Sanction::create` call in `receive_remote_sanction_notice` → STOP. ADR-006 is non-negotiable.
