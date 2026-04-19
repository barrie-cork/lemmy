# Agent B — Phase 6 Layer 2: AP object types

**Model:** opus. **Effort:** maximum — deepest reasoning, most thorough exploration.

**Scope:** Task 72 of `.claude/PRPs/plans/phase-6-federation.plan.md`. Three new AP object newtypes (`ApubSanctionNotice`, `ApubTrustAttestation`, `ApubModerationLabel`) and their protocol structs.

You are continuing a Brehon governance-fork Phase 6 layered-execution plan. Read `CLAUDE.md`, `.claude/PRPs/plans/phase-6-federation.plan.md` task 72 and patterns `AP_OBJECT_TRAIT_IMPL` + `PROTOCOL_STRUCT`. Agent A has already landed tasks 70+71 on phase-6 (migration + Diesel models). Read `.claude/decision-queue.json` DQ-6.1, 6.2 for context.

## Worktree

- Advisor has created `../brehon-fork-agent-b-phase6` on branch `agent-b-phase6` cut from `phase-6` tip (post Agent A merge).
- **First commands:**
  ```
  cd ../brehon-fork-agent-b-phase6
  git log --oneline -5
  git submodule update --init --recursive
  ls crates/email/translations/backend/ | head -3   # should list *.json files
  ```

## Task-hopper envelope

```
scripts/brehon/task-hopper.sh start 72 \
  --agent agent-b --kind cargo_check --layer 2 \
  --label "AP object types for governance" \
  --worktree "$(pwd)"
```
On commit: `complete 72 --commit-sha ...`. Per `.claude/rules/task-hopper.md`.

## Task 72 — AP object types

Create:

- `crates/apub/objects/src/protocol/governance/mod.rs` — exports
- `crates/apub/objects/src/protocol/governance/sanction_notice.rs` — `SanctionNoticeProtocol`
- `crates/apub/objects/src/protocol/governance/trust_attestation.rs` — `TrustAttestationProtocol`
- `crates/apub/objects/src/protocol/governance/moderation_label.rs` — `ModerationLabelProtocol`
- `crates/apub/objects/src/governance/mod.rs` — exports
- `crates/apub/objects/src/governance/sanction_notice.rs` — `ApubSanctionNotice` newtype + `impl Object`
- `crates/apub/objects/src/governance/trust_attestation.rs` — `ApubTrustAttestation` newtype + `impl Object`
- `crates/apub/objects/src/governance/moderation_label.rs` — `ApubModerationLabel` newtype + `impl Object` (stub — outbound only in v0)

Modify:

- `crates/apub/objects/src/lib.rs` — add `pub mod governance;` and `pub mod protocol { pub mod governance; ... }` (respect existing module shape).

### Patterns to mirror

- `crates/apub/objects/src/objects/private_message.rs:44-177` — the simplest `Object` trait impl. Your newtypes can return `Ok(None)` from `read_from_id` (v0: outbound only; no DB lookup), `Err(LemmyErrorType::NotFound.into())` from `delete`, and `false` from `is_deleted`.
- `crates/apub/objects/src/objects/comment.rs:48-242` — canonical reference for the `Object` trait's `verify` / `into_json` / `from_json` signatures.
- `crates/apub/objects/src/protocol/note.rs:30-58` — the `PROTOCOL_STRUCT` pattern: `#[skip_serializing_none]` + `#[serde(rename_all = "camelCase")]` + `#[serde(rename = "type")] kind: <Kind>` field.

### Field shapes (per plan §PROTOCOL_STRUCT)

```rust
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SanctionNoticeProtocol {
    #[serde(rename = "type")]
    pub(crate) kind: SanctionNoticeType,  // const-valued enum, 1 variant
    pub id: Url,
    pub actor: ObjectId<ApubPerson>,
    pub target: Url,
    pub action: SanctionAction,
    pub scope: SanctionScope,
    pub summary: String,                  // MUST be scrubbed upstream before serialisation
    pub published: DateTime<Utc>,
}
```

Define `SanctionNoticeType` as a single-variant enum (mirror `NoteType`), with serde-rename to the literal string `"SanctionNotice"`. Same shape for `TrustAttestationType` / `ModerationLabelType`.

### Gotchas

- **`kind` must be a single-variant enum**, not a bare string — matches `NoteType::Note` pattern in `note.rs`. Untagged dispatch in `SharedInboxActivities` depends on distinct type values.
- **`target: Url`** for v0 outbound-only. Do not implement `ObjectId<T>` dereference — noted in struct comment.
- **`ap_id` naming per ADR-012** — when wrapping a local DB row (e.g. on `from_json`), call the field `ap_id`, not `actor_id`. Codebase is already on `ap_id`; do not introduce `actor_id` anywhere.
- **`Object::verify`** — call `verify_domains_match(object.id.inner(), expected_domain)?` and return `Ok(())`. Actor authority to sanction is NOT verified in v0 (v2 concern).
- **Do not combine `-p lemmy_apub_objects` with `--features full`** per `feedback_features_full_workspace_only.md`. The crate doesn't declare the feature. Use `--workspace --features full` at merge point.

### Validate

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_apub_objects > .claude/build-task72.log 2>&1"
tail -20 .claude/build-task72.log
echo "exit: $?"
```

Exit 0 required. Also confirm `cargo check --workspace --features full` still passes — advisor will run this at merge point 1b.

## Commit

`feat(governance): task 72 — AP object types for governance` with plan-template body. One commit. Push `agent-b-phase6` when done; exit with commit SHA.

## Catch-fire triggers (stop, write DQ entry)

- `activitypub_federation::traits::Object` signature doesn't match plan's 0.7.0-beta.10 assumptions → DQ entry before guessing.
- Workspace red after your commit → roll back locally, don't force-push.
- Any need to rename a type across crates after Agent C starts → stop and wait for merge point alignment.
