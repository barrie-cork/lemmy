# Brief: m1-a BM-POLL-CR

## 1. Role + dispatch

`[role:bm-task] m1-a-bm-poll-cr — see .claude/PRPs/briefs/m1-a-bm-poll-cr-1.md`

## 2. Scope

Poll CodeRabbit findings on PR #179 (`phase-m1-a → governance-v0`) on `barrie-cork/lemmy`.

**Produce:**
- Findings YAML at `.claude/PRPs/reviews/pr-179-findings.yaml` (overwrite shell — first real poll on this PR; shell was written by bm-pr with counters at 0)
- Runlog entry in `.claude/runlog/bm-runlog.md` (create if absent)
- Summary of finding counts by bucket + severity in task output

**Do NOT:**
- Merge the PR
- Post any PR comments or submit reviews
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `tests/`, `migrations/`
- Touch `docs/` or `services/` source files

## 3. Context

CodeRabbit has completed its review on PR #179. The review is fully posted BEFORE this bm-poll-cr runs — no wait required; proceed directly to YAML authorship. Ingest ALL findings (actionable comments + any nitpick/outside-diff findings CR folded into collapsible sections) into the findings YAML.

**PR scope reminder (M1-a Tree A + Tree C — Tasks 8–14):**
- `services/bridge/` — greenfield Rust crate (workspace-EXCLUDED from Lemmy Cargo workspace). Files: `Cargo.toml`, `src/main.rs`, `src/config.rs`, `src/lib.rs`, `src/appservice.rs`, `src/puppet.rs`, `src/relay.rs`, `src/provision.rs`, `src/soft_pause.rs`, `tests/dm_round_trip.rs` (5 `#[ignore]` stubs), `registration.yaml`, `docker-compose.yml`
- `docs/brehon-law-inspired-network/06-security-and-threat-model.md` — §2.2.2 chat-plane boundary + 4 threat rows
- `docs/brehon-law-inspired-network/07-operations-and-federation.md` — §5.6 soft-pause table
- `AGPL-NOTICE.md` — bridge + Tuwunel disclosures

**R8 reminder (workspace-excluded crate):** `services/bridge/` uses `anyhow` (NOT LemmyResult), has no Diesel, no `--features full`, no `lemmy_*` imports. CR findings about Lemmy-workspace conventions (LemmyResult, actix, Diesel types) should be flagged as `bucket: rebut` candidates — they do not apply to this excluded crate.

CR findings should focus on: Rust quality in `services/bridge/src/` (error handling, the fire-and-forget style HTTP call in relay.rs, puppet map thread-safety, soft-pause poll logic), `registration.yaml` AS config correctness, docker-compose accuracy, doc quality.

## 4. Required reading

- `.claude/rules/branch-manager.md` — file-ownership boundaries, autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (stable `id`, bucket enum, severity enum)

## 5. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- PR number: **179**
- Record poll results in `.claude/runlog/bm-runlog.md`
- Do NOT post comments or submit reviews (those are separate confirm-gated verbs)
- Initial bucket: default `fix-in-pr` for CRITICAL/MAJOR; flag `rebut`/`carry-forward`/`wont-fix` candidates for advisor triage at gate-3 (do not finalize buckets — bm-triage + advisor own that)
- `source: coderabbit` on every finding; stable `id` per finding; `severity` ∈ {critical, major, medium, low, nit}
- **R8 findings flagged:** if CR flags workspace conventions (LemmyResult, actix, Diesel) on bridge code → initial bucket `rebut`, note "R8: workspace-excluded crate uses anyhow/axum"
- `poll_count: 1`, `last_poll_at: <ISO 8601>`
- `counters` block must be regenerated from the finding list (not left at bm-pr shell zeros)
