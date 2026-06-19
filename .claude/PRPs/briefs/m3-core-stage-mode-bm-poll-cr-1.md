# Brief: m3-core-stage-mode BM-POLL-CR

## 1. Role + dispatch

`[role:bm-task] m3-core-stage-mode bm-poll-cr — see .claude/PRPs/briefs/m3-core-stage-mode-bm-poll-cr-1.md`

## 2. Scope

Poll CodeRabbit + Copilot findings on PR #202 (`phase-m3-core-stage-mode → governance-v0`) on `barrie-cork/lemmy`.

**Produce:**
- Findings YAML at `.claude/PRPs/reviews/pr-202-findings.yaml` (overwrite shell — first real poll on this PR)
- Runlog entry in `.claude/runlog/bm-runlog.md` (create if absent)
- Summary of finding counts by bucket + severity in task output

**Do NOT:**
- Merge the PR
- Post any PR comments or submit reviews
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `tests/`, `migrations/`, `services/` source files, `docs/`

## 3. Context

PR #202 is the M3 town-hall **stage-mode** delivery (Phase 3, bridge-side). Both reviewers should have completed:
- **CodeRabbit:** ingest all inline review comments (`gh api repos/barrie-cork/lemmy/pulls/202/comments`) + any nitpick / outside-diff findings CR placed in collapsible sections of its top-level comment (`gh pr view 202 --repo barrie-cork/lemmy --comments`). If CR has NOT posted yet (zero CR comments AND PR open <30 min), STOP and write a `kind: blocker` DQ noting "CR not yet posted on #202" rather than authoring an empty YAML.
- **Copilot:** if `copilot-pull-request-reviewer` posted a review, ingest its findings too.

Per SCHEMA.md `source` ∈ {coderabbit, claude, user} (read SCHEMA.md first — if a `copilot` value has since been added, use it): tag CR findings `source: coderabbit`; tag Copilot findings `source: claude` with a body note "(copilot-pull-request-reviewer)".

**PR scope reminder (m3-core-stage-mode — chair-controlled stage mode, Tasks 1-6):**
- `crates/api/api_common/src/governance.rs` + `crates/api/api/src/governance/governance_log.rs` — `RoomEventPayload` 5 optional chair-action fields + `CaseTransitionEvent.chair_pseudonym` (Task 1)
- `crates/api/api_utils/src/bridge_notify.rs` — `chair_pseudonym: None` initialiser (Task 1)
- `services/bridge/src/livekit_jwt.rs` — `mint_access_token(can_publish)` presenter/watcher grant (Task 2)
- `services/bridge/src/stage.rs` — NEW: chair seat + FIFO + mic-pass state machine + 30s grace + emit-intent seam (Tasks 3,4,5)
- `services/bridge/src/bridge_room.rs` — queue_state/chair_id accessors (Task 3)
- `services/bridge/src/room_event_client.rs` — NEW: bridge→binary POST + chair-action emitters + drain_emits (Tasks 5,6)
- `services/bridge/src/config.rs` — brehon_room_event_url /api/v4 fix (Task 5)
- `services/bridge/src/room_provisioner.rs` — stage-mode provisioning + Q&A sidebar + drain callsite (Task 6)
- `services/bridge/tests/stage_mode.rs` — NEW: docker-gated #[ignore] e2e stub (Task 6)

**Key ADR constraints (for CR finding evaluation — do NOT finalize buckets, just flag):**
- ADR-015: all chair-action chain payload fields are pseudonyms (`*_pseudonym`); `chair_id` seated from pseudonyms; NO person_id/username/MXID into a chain entry
- ADR-016: Q&A sidebar = Matrix text timeline (never hashed/POSTed); `post_room_event` carries metadata only
- Mic = LiveKit publish-grant re-mint, NOT Matrix power-levels

## 4. Required reading

- `.claude/rules/branch-manager.md` — file-ownership boundaries, autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (stable `id`, bucket enum, severity enum, source enum — check for `copilot`)

## 5. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- PR number: **202**
- Record poll results in `.claude/runlog/bm-runlog.md`
- Do NOT post comments or submit reviews (separate confirm-gated verbs)
- Initial bucket: default `fix-in-pr` for CRITICAL/MAJOR; flag `rebut`/`carry-forward`/`wont-fix` candidates for advisor triage at gate-3 (do not finalize buckets — bm-triage + advisor own that)
- Stable `id` per finding; `severity` ∈ {critical, major, medium, low, nit}
- `poll_count: 1`, `last_poll_at: <ISO 8601>`
- `counters` block regenerated from the finding list (not bm-pr shell zeros)
- Ingest ALL findings from BOTH reviewers — note the reviewer per finding so the advisor's gate-3 triage can weigh source
- **Bridge-only code = no e2e fragility.** This PR's source is `services/bridge/**` + additive DTO fields; CR findings are likely doc/style/idiom, not the e2e-edit-hang class.
