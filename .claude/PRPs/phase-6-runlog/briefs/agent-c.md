# Agent C — Phase 6 Layer 2: AP activity types

**Model:** opus. **Effort:** maximum — deepest reasoning, most thorough exploration.

**Scope:** Task 73 of `.claude/PRPs/plans/phase-6-federation.plan.md`. Three Create-wrapper activities (`PublishSanctionNotice`, `PublishTrustAttestation`, `PublishLabel`), their protocol structs, and registration in `SharedInboxActivities`.

You are continuing a Brehon governance-fork Phase 6 layered-execution plan. Read `CLAUDE.md`, plan task 73 and patterns `AP_ACTIVITY_TRAIT_IMPL` + `ACTIVITY_ENUM_REGISTRATION`. Agent B has just shipped task 72 (AP objects) on phase-6.

## Worktree

- Advisor has created `../brehon-fork-agent-c-phase6` on branch `agent-c-phase6` cut from `phase-6` tip (post Agent B merge).
- **First commands:** `cd ../brehon-fork-agent-c-phase6 && git log --oneline -5 && git submodule update --init --recursive`. Confirm `crates/apub/objects/src/governance/` exists (Agent B's commit).

## Task-hopper envelope

```bash
scripts/brehon/task-hopper.sh start 73 \
  --agent agent-c --kind cargo_check --layer 2 \
  --label "AP activities for governance" \
  --worktree "$(pwd)"
```

## Task 73 — AP activity types

Create:

- `crates/apub/activities/src/protocol/governance/mod.rs`
- `crates/apub/activities/src/protocol/governance/publish_sanction_notice.rs` — `PublishSanctionNoticeProtocol` (serde struct wrapping the object)
- `crates/apub/activities/src/protocol/governance/publish_trust_attestation.rs`
- `crates/apub/activities/src/protocol/governance/publish_label.rs`
- `crates/apub/activities/src/governance/mod.rs`
- `crates/apub/activities/src/governance/publish_sanction_notice.rs` — `PublishSanctionNotice` struct + `impl Activity`
- `crates/apub/activities/src/governance/publish_trust_attestation.rs` — same pattern
- `crates/apub/activities/src/governance/publish_label.rs` — stub: type + trait impl, `verify`/`receive` bodies = `Ok(())`

Modify:

- `crates/apub/activities/src/lib.rs` — add `pub mod governance;` + `pub mod protocol { pub mod governance; ... }`.
- `crates/apub/activities/src/activity_lists.rs` — add `PublishSanctionNotice`, `PublishTrustAttestation`, `PublishLabel` variants to `SharedInboxActivities`. **Insert BEFORE** `RawAnnouncableActivities` (catch-all must remain last).

### Patterns to mirror

- `crates/apub/activities/src/community/report.rs:51-182` — closest analogy for a single-target moderation-adjacent Create activity. `Report::send` at lines 62-74 is the structural reference (though your `send` helper lives in task 74, not here).
- `crates/apub/activities/src/block/block_user.rs:39-206` — the `#[async_trait::async_trait] impl Activity for ...` canonical shape.
- `crates/apub/activities/src/activity_lists.rs:38-98` — `SharedInboxActivities` enum pattern, `#[serde(untagged)]` + `#[enum_delegate::implement(Activity)]`.

### Trait shape

```rust
#[async_trait::async_trait]
impl Activity for PublishSanctionNotice {
    type DataType = LemmyContext;
    type Error = LemmyError;

    fn id(&self) -> &Url { &self.id }
    fn actor(&self) -> &Url { self.actor.inner() }

    async fn verify(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
        verify_is_public(&self.to, &self.cc)?;
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
        // STUB ONLY — Agent E (task 75) fills this with receive_remote_sanction_notice.
        // TODO(task75): wire receive_remote_sanction_notice
        Ok(())
    }
}
```

This stub-and-wire split is **intentional** per plan §task 73 gotcha — lets task 74 and task 75 proceed without file conflicts in Layer 3.

### Gotchas

- **Variant ordering in `SharedInboxActivities`** — `#[serde(untagged)]` tries variants in declaration order. New variants go **before** `RawAnnouncableActivities` (catch-all). The three new variants have distinct `type` fields so no overlap, but ordering matters for structural fallthrough.
- **Activity id generation** — use `generate_activity_id(kind, context)` helper (see `community/report.rs:60`). Do not roll your own.
- **`PublishLabel` stub** — empty `verify` (`Ok(())`) and empty `receive` is acceptable in v0; it's not wired from any handler.
- **Do not write the `send_local_sanction_notice` helper here** — that's task 74. This task defines types and trait impls only.
- **Do not combine `-p lemmy_apub_activities` with `--features full`**.

### Validate

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub_activities > .claude/build-task73.log 2>&1"
tail -20 .claude/build-task73.log
echo "check exit: $?"

cmd //c "scripts\\brehon\\cargo-clippy.bat -p lemmy_apub_activities --no-deps -- -D warnings > .claude/clippy-task73.log 2>&1"
tail -20 .claude/clippy-task73.log
echo "clippy exit: $?"
```

Both exit 0 required.

## Commit

`feat(governance): task 73 — AP activities for governance` with plan-template body. Push `agent-c-phase6` when done.

## Catch-fire triggers

- `#[enum_delegate::implement(Activity)]` macro surfaces an API drift → DQ entry; the enum_delegate version pinned is 0.2.0.
- `activity_lists.rs` rebuild hits a derive-overlap error → DQ entry (means variant ordering or overlapping `type` field shapes).
