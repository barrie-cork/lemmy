# Brief: m3-core-recording BM-POLL-CR

## 1. Role + dispatch

`[role:bm-task] m3-core-recording bm-poll-cr — see .claude/PRPs/briefs/m3-core-recording-bm-poll-cr-1.md`

## 2. Scope

Poll CodeRabbit + Copilot findings on PR #205 (`phase-m3-core-recording → governance-v0`) on `barrie-cork/lemmy`.

**Produce:**
- Findings YAML at `.claude/PRPs/reviews/pr-205-findings.yaml` (create — first poll on this PR)
- Runlog entry appended to `.claude/runlog/m3-core-recording-runlog.md` (create if absent)
- Summary of finding counts by bucket + severity in task output

**Do NOT:**
- Merge the PR
- Post any PR comments or submit reviews
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `tests/`, `migrations/`, `services/bridge/src/`, `docs/`

## 3. Context

PR #205 is the M3 town-hall **optional recording** delivery (Phase 5, bridge-side). CodeRabbit has posted its full review (1 review `COMMENTED`, walkthrough comment, **5 inline review-thread comments** observed at poll time).

**Ingest both reviewers:**
- **CodeRabbit:** all inline review comments (`gh api repos/barrie-cork/lemmy/pulls/205/comments`) + any findings in collapsible sections of the top-level CR walkthrough comment (`gh pr view 205 --repo barrie-cork/lemmy --comments`). If CR has NOT posted (zero CR comments AND PR open <30 min), write a `kind: blocker` DQ instead of an empty YAML. (CR HAS posted — ingest the 5 inline findings.)
- **Copilot:** if `copilot-pull-request-reviewer` posted a review, ingest its findings too.

Per SCHEMA.md: tag CR findings `source: coderabbit`; Copilot findings `source: claude` with body note "(copilot-pull-request-reviewer)".

**PR scope (m3-core-recording — Tasks 1-6):**
- `crates/api/api/src/governance/governance_log.rs` — `RoomEventPayload` 5 optional recording fields `{media_url, content_sha256, duration_s, speakers, attendance_count}` (Task 1)
- `services/bridge/src/config.rs` — 4 S3 settings (Task 2)
- `services/bridge/src/bridge_room.rs` — `read_recording_config`/`write_recording_config`/`record_town_halls_enabled` helpers (Task 2)
- `services/bridge/src/recording.rs` — NEW: `compute_content_sha256` + `RecordingSink` trait + `maybe_record` flag-gate + `is_participant` floor (Tasks 3-5)
- `services/bridge/src/stage.rs` — `record_uploaded` EmitIntent push (Task 4)
- `services/bridge/src/room_event_client.rs` — `RoomEventPayload` 5-field mirror (Task 4)
- `services/bridge/src/appservice.rs` — `/brehon/recording/{id}` route + `handle_recording_fetch` calling `is_participant` before serving (Task 5)
- `services/bridge/src/main.rs` — `mod recording;` (Task 3)
- `services/bridge/tests/recording.rs` — NEW: 3 `#[ignore]` docker-gated e2e stubs, `todo!()` bodies (Task 6)
- `Cargo.toml`/`Cargo.lock` — the one new dep `rust-s3` (Task 3)

**Key ADR constraints (for finding evaluation — do NOT finalize buckets, just flag):**
- ADR-015: the participant-floor (`is_participant`) + `speakers` + `actor_pseudonym` are PSEUDONYMS; the fetch handler MUST call `is_participant` BEFORE serving (non-participant → 403); the floor cannot be zero. Any CR finding on the requester-pseudonym source / the participant-floor wiring / empty-set handling is ON the ADR-015 authz path — flag for advisor eyes at gate-3.
- ADR-016: `room_recording_uploaded` payload = metadata only `{media_url, content_sha256, duration_s, speakers, attendance_count}` + top-level `actor_pseudonym`; the MP4 bytes go to S3, never the chain. R11: `content_sha256` reaches the chain ONLY via `append_room_event` (no side-channel write).
- The participant-set + requester-pseudonym resolution is **scaffold-grade this phase** (Phase-6 wires live session-auth) — CR findings claiming "requester pseudonym is trusted/spoofable" or "participant set is stubbed" are EXPECTED scaffold limitations; flag as `carry-forward` candidates (Phase-6), NOT `fix-in-pr`, for advisor decision at gate-3.

## 4. Required reading

- `.claude/rules/branch-manager.md` — file-ownership boundaries, autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (stable `id`, bucket enum, severity enum, source enum)

## 5. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- PR number: **205**
- Append poll results to `.claude/runlog/m3-core-recording-runlog.md` (create if absent — first BM runlog entry for this phase)
- Do NOT post comments or submit reviews
- Initial bucket: default `fix-in-pr` for CRITICAL/MAJOR; flag `rebut`/`carry-forward`/`wont-fix` candidates for advisor triage at gate-3 (do not finalize buckets). The scaffold-grade requester-pseudonym + participant-set findings are likely `carry-forward` (Phase-6) — flag, don't finalize.
- Stable `id` per finding; `severity` ∈ {critical, major, medium, low, nit}
- `poll_count: 1`, `last_poll_at: <ISO 8601>`
- `counters` block regenerated from the finding list
- Ingest ALL findings from BOTH reviewers — note reviewer per finding
- **Bridge + stub code = no e2e fragility.** `recording.rs` integration tests are `todo!()` stubs; CR findings on them are likely doc/style. The deterministic unit tests are unit-only (no Docker). CR findings on the stub `todo!()` bodies or `#[ignore]` annotations need advisor eyes at gate-3 before any fix-in-pr action.
- **One CR finding flags `.claude/decision-queue.json:8591` ("Task 6 decision-queue entry missing")** — this is likely STALE (the advisor already resolved the Task 6 validate-pending DQ `0aa481cce3a6-001` after CR snapshotted the PR). Ingest it as a finding but flag it `rebut` candidate (already-resolved) for advisor confirmation at gate-3.
