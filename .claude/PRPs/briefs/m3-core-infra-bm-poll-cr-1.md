# Brief: m3-core-infra BM-POLL-CR

## 1. Role + dispatch

`[role:bm-task] m3-core-infra bm-poll-cr — see .claude/PRPs/briefs/m3-core-infra-bm-poll-cr-1.md`

## 2. Scope

Poll CodeRabbit + Copilot findings on PR #201 (`phase-m3-core-infra → governance-v0`) on `barrie-cork/lemmy`.

**Produce:**
- Findings YAML at `.claude/PRPs/reviews/pr-201-findings.yaml` (overwrite shell — first real poll on this PR)
- Runlog entry in `.claude/runlog/bm-runlog.md` (create if absent)
- Summary of finding counts by bucket + severity in task output

**Do NOT:**
- Merge the PR
- Post any PR comments or submit reviews
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `tests/`, `migrations/`, `services/` source files, `docs/`

## 3. Context

Both reviewers have completed on PR #201 — proceed directly to YAML authorship, no wait:
- **CodeRabbit:** review complete, **8 inline review comments** (`gh api repos/barrie-cork/lemmy/pulls/201/comments`). Also fold in any nitpick / outside-diff findings CR placed in collapsible sections of its top-level comment.
- **Copilot:** one review posted (`copilot-pull-request-reviewer`) — ingest its findings too, `source: claude` is wrong; use `source: coderabbit` for CR and `source: claude` only for advisor-authored. For Copilot findings use `source: coderabbit` is also wrong — **add Copilot findings with a `reviewer: copilot` note in the finding body and `source: coderabbit`** is NOT correct either. Per SCHEMA.md `source` ∈ {coderabbit, claude, user}: tag CR findings `source: coderabbit`; tag Copilot findings `source: coderabbit` is disallowed → use `source: claude` with body note "(copilot-pull-request-reviewer)". (If SCHEMA.md has since added a `copilot` source value, prefer that — read SCHEMA.md first.)

**PR scope reminder (m3-core-infra — RTC stack deployable+optional, Tasks 1-6):**
- `migrations/2026-06-18-000000-0000_seed_rtc_enabled_config/` — rtc_enabled config seed (data-only)
- `crates/api/api/src/governance/bridge_read.rs` — `BridgeStatus.rtc_enabled` + `get_bridge_actor_pseudonym` endpoint
- `crates/api/routes/src/lib.rs` — `/bridge/actor-pseudonym` route
- `crates/server/tests/e2e/governance.rs` — m3_actor_pseudonym + m3_rtc_disabled clean-posture tests
- `services/bridge/src/{bridge_room,livekit_jwt,config,main}.rs` — RTC cols, JWT mint, config
- `services/bridge/docker-compose.yml` — RTC sidecars (profiles:[rtc])
- `AGPL-NOTICE.md` — RTC stack rows

**Key ADR constraints (for CR finding evaluation — do NOT finalize buckets, just flag):**
- ADR-015: `actor_pseudonym` in all bridge payloads; `BridgeActorPseudonym` response = pseudonym ONLY (no person_id/username/email); JWT `sub` = pseudonym
- ADR-011: AGPL-NOTICE rows mandatory for the 3 sidecars (Element Call AGPL-3.0)
- ADR-013/016: rtc_enabled default false (clean posture)

## 4. Required reading

- `.claude/rules/branch-manager.md` — file-ownership boundaries, autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (stable `id`, bucket enum, severity enum, **source enum** — check whether a `copilot` source value exists)

## 5. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- PR number: **201**
- Record poll results in `.claude/runlog/bm-runlog.md`
- Do NOT post comments or submit reviews (separate confirm-gated verbs)
- Initial bucket: default `fix-in-pr` for CRITICAL/MAJOR; flag `rebut`/`carry-forward`/`wont-fix` candidates for advisor triage at gate-3 (do not finalize buckets — bm-triage + advisor own that)
- Stable `id` per finding; `severity` ∈ {critical, major, medium, low, nit}
- `poll_count: 1`, `last_poll_at: <ISO 8601>`
- `counters` block regenerated from the finding list (not bm-pr shell zeros)
- Ingest ALL findings from BOTH reviewers — note the reviewer per finding so the advisor's gate-3 triage can weigh source
