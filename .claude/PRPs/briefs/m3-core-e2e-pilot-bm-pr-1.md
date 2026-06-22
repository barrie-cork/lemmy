---
role: bm-task
verb: bm-pr
phase: m3-core-e2e-pilot
base_branch: governance-v0
created: 2026-06-22
---

# bm-task brief — m3-core-e2e-pilot bm-pr

**Role:** `[role:bm-task]`
**Verb:** `bm-pr`
**Phase:** `m3-core-e2e-pilot`
**Base branch:** `governance-v0`

## Dispatch line

```
[role:bm-task] m3-core-e2e-pilot bm-pr — see .claude/PRPs/briefs/m3-core-e2e-pilot-bm-pr-1.md
```

## Scope

Open a PR from `phase-m3-core-e2e-pilot` into `governance-v0` on `barrie-cork/lemmy`.

- Title: `feat(e2e,bridge): M3-core e2e acceptance harness — live docker-compose suite (stage-mode, recording, emergency-mute, room-provisioning) + rtc_enabled wire-contract gate`
- NOT a draft (CodeRabbit must review)
- `--repo barrie-cork/lemmy` mandatory
- PR-open only — NO code edits

## PR body

```
## Summary

The M3-core **e2e acceptance harness** (Phase 6/6, FINAL): a live docker-compose
integration-test suite that exercises the M3-core town-hall RTC features end-to-end
against real services (Tuwunel Matrix homeservers, LiveKit, MinIO S3, the bridge),
plus one small bridge↔governance wire-contract fix the live gate exposed. All four
test targets are green on the local/daemon stack (gate-4 = LOCAL, advisor-laptop run);
the suite was hardened across five fix cycles, the last of which (cycle-5) fixed the
three defects the first full live in-stack run surfaced.

**The four e2e targets (all green, advisor-laptop validated 2026-06-22):**
- **stage_mode** — `four_mic_pass_then_grace_boundary_emits_chair_entries` (4-user mic-pass + grace boundary + chair override/transfer).
- **recording** — 3/3: `recording_lands_with_hash_on_chain` (MP4→MinIO→content_sha256→chain), `clean_posture_no_recording_when_disabled`, `participant_floor_fetch`.
- **emergency_mute** — `mute_all_drops_all_publishers_cross_instance_under_500ms` (<500ms in-instance publisher-client revoke; cross-instance zero-holder).
- **room_provisioning** — `rtc_disabled_townhall_clean_posture` (R7 negative invariant) + `anonymous_townhall_identity_never_reaches_livekit`; 5 `todo!()` stubs D2-pilot-deferred.

**Harness (the docker-compose stack):**
- Real **Tuwunel** Matrix homeservers (base + federated 2nd instance, distinct server_names — BUG-15 preserved) replacing matrix-conduit v0.6.0, which never auto-registered the bridge appservice → createRoom 401. Tuwunel auto-loads `registration.yaml` from its `appservice_dir`.
- **MinIO** S3 + a `minio-init` (mc) one-shot that creates the `recordings` bucket (MinIO does not auto-create).
- **LiveKit** v1.7 with explicit `--keys "devkey: devsecret"` (--dev does NOT set keys; its placeholder secret is "secret", mismatching the devsecret-signed JWTs).
- A `room-event-stub` (caddy respond :3000, 200) so the recording on-chain POST gets a 2xx in the base stack (chain persistence is the governance suite's gate, not this bridge integration test).

**The one code fix beyond the harness (cycle-5 defect 2 — bridge↔governance wire contract):**
- `rtc_enabled: Option<bool>` added to `CaseTransitionEvent` on BOTH ends (governance `crates/api/api_common/src/governance.rs` + the bridge mirror `services/bridge/src/room_provisioner.rs`), populated from the `rtc_enabled` governance config by the emitter (`crates/api/api_utils/src/bridge_notify.rs`), and consumed by a per-event gate short-circuit in `provision_townhall_stage_room` (`if event.rtc_enabled == Some(false) { return }` — before the creds gate). This makes the RTC stage-seat decision per-CASE (criterion 146 / R7) instead of stack-wide-creds-only. Additive field (`#[serde(default)]`, default-ON for back-compat); ADR-016 additive (no new ADR — it's an additive field to the M2 reference integration). The two other cycle-5 fixes (recording :3000 stub, emergency_mute 200ms fast-fail timeout) are harness/test-only.

**Validation:** `cargo check --workspace --features full` EXIT 0 (full lemmy workspace compiles with the wire-contract field). All four e2e targets green on the live stack. `/brehon-verify` 3/3 cycle-5 stories ✓. Gate-4 LOCAL (advisor-laptop), not Shape G.

## Plan reference

`.claude/PRPs/plans/m3-core-e2e-pilot-e2e-fixes.plan.md` (cycle-5) + the parent
`.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` (the e2e suite) +
`.claude/PRPs/plans/m3-core-e2e-pilot-as-register.plan.md` (Tuwunel swap).
Verify: `.claude/PRPs/reports/m3-core-e2e-pilot-verify.md`. Retro: `.claude/PRPs/reports/m3-core-e2e-pilot-e2e-fixes-retro.md`.

## Commit log (phase diff)

See `git log governance-v0..phase-m3-core-e2e-pilot --oneline`
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
- Body assembles from this brief + `git log governance-v0..phase-m3-core-e2e-pilot --oneline`
- If a PR already exists on this branch (`gh pr view`), fall through to `gh pr edit --body-file` instead of `gh pr create`
- After opening, capture the PR number + URL in the runlog + the HANDOVER block below
```

## HANDOVER

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-bm-pr
  pr_number: <N>
  pr_url: <url>
  base: governance-v0
  head: phase-m3-core-e2e-pilot
  draft: false
  repo: barrie-cork/lemmy
  notes: "<confirm base=governance-v0 not main; confirm not draft; confirm --repo flag used>"
```
