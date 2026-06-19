# Brief: m3-core-emergency-mute BM-POLL-CR

## 1. Role + dispatch

`[role:bm-task] m3-core-emergency-mute bm-poll-cr — see .claude/PRPs/briefs/m3-core-emergency-mute-bm-poll-cr-1.md`

## 2. Scope

Poll CodeRabbit + Copilot findings on PR #204 (`phase-m3-core-emergency-mute → governance-v0`) on `barrie-cork/lemmy`.

**Produce:**
- Findings YAML at `.claude/PRPs/reviews/pr-204-findings.yaml` (create — first poll on this PR)
- Runlog entry appended to `.claude/runlog/m3-core-emergency-mute-runlog.md`
- Summary of finding counts by bucket + severity in task output

**Do NOT:**
- Merge the PR
- Post any PR comments or submit reviews
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `tests/`, `migrations/`, `services/bridge/src/`, `docs/`

## 3. Context

PR #204 is the M3 town-hall **emergency mute-all** delivery (Phase 4, bridge-side). CodeRabbit has posted (1 review, 1 comment observed at poll time).

**Ingest both reviewers:**
- **CodeRabbit:** all inline review comments (`gh api repos/barrie-cork/lemmy/pulls/204/comments`) + any findings in collapsible sections of the top-level CR comment (`gh pr view 204 --repo barrie-cork/lemmy --comments`). If CR has NOT posted (zero CR comments AND PR open <30 min), write a `kind: blocker` DQ instead of an empty YAML.
- **Copilot:** if `copilot-pull-request-reviewer` posted a review, ingest its findings too.

Per SCHEMA.md: tag CR findings `source: coderabbit`; Copilot findings `source: claude` with body note "(copilot-pull-request-reviewer)".

**PR scope (m3-core-emergency-mute — Tasks 1-4 + bundled clippy-debt fix):**
- `crates/api/api/src/governance/governance_log.rs` — `RoomEventPayload.federated: Option<bool>` (Task 1)
- `services/bridge/src/mute_handler.rs` — NEW: `compute_mute_all_override` (pure) + `mute_all_power_levels` (async, Task 2)
- `services/bridge/src/sanction_handler.rs` — `get_power_levels`/`put_power_levels` widened to `pub(crate)` (Task 2)
- `services/bridge/src/stage.rs` — `pub fn mute_all(publishers, federated, sink)` + marquee unit test (Task 3)
- `services/bridge/src/room_event_client.rs` — `federated: Option<bool>` mirror field + test (Task 3)
- `services/bridge/tests/emergency_mute.rs` — NEW: `#[ignore]` docker-gated e2e stub, `todo!()` body (Task 4)
- 7 governance/bridge files — pre-existing clippy-baseline-debt clear (20 lints; user-authorised): `actor_app_link.rs`, `bridge_auth.rs`, `messaging_config.rs`, `sanction_publisher.rs`, `state.rs`, `submit_jury_vote.rs`, `revoke_endorsement.rs`

**Key ADR constraints (for finding evaluation — do NOT finalize buckets, just flag):**
- ADR-015: all `room_mute_all` chain payload fields are pseudonyms (`actor_pseudonym`); no `person_id`/MXID/username into any chain entry
- ADR-016: `room_mute_all` payload = metadata only (`{ federated, actor_pseudonym }`) — NOT content; the emoji/mute signal never hashed
- `bridge_auth.rs` clippy fix is a real security change (`?` propagation on unset `BRIDGE_CALLBACK_SECRET`) — any finding claiming it's "unnecessary" should be flagged for advisor rebut consideration

## 4. Required reading

- `.claude/rules/branch-manager.md` — file-ownership boundaries, autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (stable `id`, bucket enum, severity enum, source enum)

## 5. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- PR number: **204**
- Append poll results to `.claude/runlog/m3-core-emergency-mute-runlog.md` (file exists — append, don't overwrite)
- Do NOT post comments or submit reviews
- Initial bucket: default `fix-in-pr` for CRITICAL/MAJOR; flag `rebut`/`carry-forward`/`wont-fix` candidates for advisor triage at gate-3 (do not finalize buckets)
- Stable `id` per finding; `severity` ∈ {critical, major, medium, low, nit}
- `poll_count: 1`, `last_poll_at: <ISO 8601>`
- `counters` block regenerated from the finding list
- Ingest ALL findings from BOTH reviewers — note reviewer per finding
- **Bridge + stub code = no e2e fragility.** `emergency_mute.rs` is a `todo!()` stub; CR findings on it are likely doc/style. The `mute_all` marquee test is unit-only (no Docker). CR findings on the stub's `todo!()` body or the `#[ignore]` annotation need advisor eyes at gate-3 before any fix-in-pr action.
