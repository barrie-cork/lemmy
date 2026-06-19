---
role: bm-task
verb: bm-pr
phase: m3-core-emergency-mute
base_branch: governance-v0
created: 2026-06-19
---

# bm-task brief — m3-core-emergency-mute bm-pr

**Role:** `[role:bm-task]`
**Verb:** `bm-pr`
**Phase:** `m3-core-emergency-mute`
**Base branch:** `governance-v0`

## Dispatch line

```
[role:bm-task] m3-core-emergency-mute bm-pr — see .claude/PRPs/briefs/m3-core-emergency-mute-bm-pr-1.md
```

## Scope

Open a PR from `phase-m3-core-emergency-mute` into `governance-v0` on `barrie-cork/lemmy`.

- Title: `feat(rtc,bridge): M3 town-hall emergency mute-all — cross-instance Matrix power-levels + local LiveKit revoke sweep, FIRST room_mute_all chain emission`
- NOT a draft (CodeRabbit must review)
- `--repo barrie-cork/lemmy` mandatory

## PR body

```
## Summary

Adds **federation-wide emergency mute-all** to M3 town-hall rooms (Phase 4, bridge-side): a chair-fired mute that drops EVERY non-chair publisher across federated instances. Two complementary surfaces — (a) a cross-instance **Matrix `m.room.power_levels` PUT** that raises the publish threshold above `users_default` (one PUT federates to remote instances via Matrix; OQ-V2-06), and (b) a local **LiveKit `RevokePublish` sweep** (`Stage::mute_all`) for instant in-instance effect (<500ms). Emits the **FIRST `room_mute_all`** governance-log chain entry from the bridge (the const was registered in Phase 2; this is emit-only — entry-kind registry stays 72). All chain payloads are pseudonyms-only (ADR-015); `room_mute_all` carries metadata only — `{ federated, actor_pseudonym }` (ADR-016). The deterministic DoD is unit tests (the cr-4 zero-holder negative invariant); the docker-gated cross-instance end-to-end is an `#[ignore]`'d compile-gated stub (Phase-6 pilot grade).

- **Binary `RoomEventPayload.federated: Option<bool>`** (`governance_log.rs`): one optional `#[serde(default, skip_serializing_if = "Option::is_none")]` field so the `room_mute_all` chain entry carries the federation-wide flag; non-breaking (skip-if-none keeps existing room emissions byte-identical); `#[cfg(test)]` `room_mute_all` case asserts present-when-Some + omitted-when-None (Task 1).
- **Cross-instance power-level mute** (`mute_handler.rs` NEW): `compute_mute_all_override` (pure — raises `events["m.call.member"]` + the MSC3401 alias ABOVE `users_default`, preserving the whole power-levels object for a full PUT) + `mute_all_power_levels` (per-room `lookup_by_case` → GET → mutate → PUT, mirror `sanction_handler`); `sanction_handler.rs` `get_power_levels`/`put_power_levels` widened to `pub(crate)` (Task 2).
- **Local LiveKit revoke sweep + emission** (`stage.rs::mute_all`): takes the EXPLICIT locally-known publisher slice, `RevokePublish` for EVERY one (zero-holder invariant — no surviving holder), clears `current`, pushes exactly one `room_mute_all` `EmitIntent` with `actor_pseudonym = chair` + `federated = Some(bool)`. The marquee unit test `mute_all_revokes_all_publishers` asserts set-equality of revokes (NOT "≥1 fired") with a delete-the-revoke mechanical check — the cr-4 zero-holder negative invariant (Task 3).
- **Bridge `RoomEventPayload.federated` mirror** (`room_event_client.rs`): mirror field + `mute_all_request_json_shape` test (entry_kind `room_mute_all`, `federated == true`, chair-action fields omitted) (Task 3).
- **Docker-gated cross-instance e2e stub** (`services/bridge/tests/emergency_mute.rs` NEW): `#[ignore]` compile-gated end-to-end asserting all publishers dropped <500ms **measured at the publisher client** across two federated instances + a `room_mute_all` chain row with chair pseudonym + `federated: true`; `todo!()` body (Phase-6 pilot grade); the PRD cross-instance-SLA fallback is documented-not-dropped (Task 4).
- **Bundled: pre-existing governance clippy-baseline-debt clear** (7 files: `actor_app_link.rs`, `bridge_auth.rs`, `messaging_config.rs`, `sanction_publisher.rs`, `state.rs`, `submit_jury_vote.rs`, `revoke_endorsement.rs`): 20 `-D warnings` lints that Task 1's `--workspace` clippy unmasked (pre-existing baseline, not introduced this phase); user-authorised inline fix. `bridge_auth.rs` got a real `?` fix (hard-error on unset `BRIDGE_CALLBACK_SECRET` — a security boundary); the rest are mechanical (`#[expect]` with rationale, `let`-chains, `&context`). Workspace clippy now green.

## Validation

- **Bridge Linux-compile gate** (`cargo-linux.sh check + clippy --no-deps -- -D warnings`, Docker rust:1.95): EXIT 0 @ phase tip (advisor-laptop). check exit 0; clippy 0 lints. DQ `4c8592a9c25f-001` (Task 3) + `fc785e35b108-001` (Task 4) both `result: pass`.
- **Bridge unit tests** (`cargo-linux.sh test --bins`): `mute_all_revokes_all_publishers` 1 passed (cr-4 zero-holder set-equality, N=4 explicit slice); `room_event_client::tests` 3 passed (incl `mute_all_request_json_shape`); `compute_mute_all_override_cases` passed (Cohort A, Task 2).
- **Integration-test compile** (`cargo-linux.sh test --test emergency_mute --no-run`): EXIT 0 — `emergency_mute.rs` `#[ignore]` stub compiles (Executable built; live run is Phase-6 pilot).
- **crates static** (Task 1, Windows `--workspace --features full`): check + clippy EXIT 0 (`federated`-field serde roundtrip; + the bundled debt-fix made `--workspace` clippy green).
- **No migration** (no new DB column; `room_mute_all` const shipped Phase 2). **No new dep.** **Registry count UNCHANGED at 72** (emit-only).

## Plan reference

`.claude/PRPs/plans/m3-core-emergency-mute.plan.md` — Tasks 1–4 + §16a Stories 1–4.

## Commit log (phase diff)

See `git log governance-v0..phase-m3-core-emergency-mute --oneline`
```

## Required reading

- `.claude/rules/branch-manager.md`
- `.claude/rules/gh-pr-fork-target.md`
- `.claude/commands/bm/bm-pr.md`

## Constraints

- Base MUST be `governance-v0` (never `main`)
- `--repo barrie-cork/lemmy` on all `gh pr` commands
- NOT a draft (CodeRabbit skips drafts)
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`, `services/bridge/**`, `docs/**`, plan/PRD files — PR-open only, no code edits
- Body assembles from this brief + `git log governance-v0..phase-m3-core-emergency-mute --oneline`
- If a PR already exists on this branch (`gh pr view`), fall through to `gh pr edit --body-file` instead of `gh pr create`
