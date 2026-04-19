# v1 Issue Triage — Phase 6 Parallel Workability

**Date:** 2026-04-19
**Author:** triage-claude (Opus 4.7, 1M ctx)
**Audience:** advisor + user, deciding what to plan/ship while Phase 6 federation runs in the advisor worktree
**Inputs:** open `v1` issues #11–#31 (excluding plan-drift #6); Phase 6 plan at
`C:\Users\barri\Developer\brehon-fork-advisor-phase6\.claude\PRPs\plans\phase-6-federation.plan.md`;
`docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` §3 Phase 6;
`99-decisions-and-open-questions.md` OQ-018, OQ-019, OQ-025
**Phase 6 modified-files cross-reference:** plan §"Files to Change" → MODIFIED FILES table (lines 430–447)

---

## §1 Executive Summary

**Bucket counts (19 issues triaged):**

| Bucket | Count | Issue numbers |
|---|---:|---|
| **A. Plan-and-ship now** (zero conflict, no design blocker) | **10** | #20, #21, #22, #23, #24, #25, #26, #27, #28, #31 |
| **B. Plan now, ship after Phase 6 merges** (file-overlap rebase risk) | **4** | #19, #29, #30 + a half-credit on #16 (see §4) |
| **C. Wait — needs design/PRD first** | **5** | #11, #14, #15, #16, #19 (see notes) |
| **D. Wait — direct Phase 6 conflict** | **0** | (see §4 watch-list — #19 and #29 sit on the boundary) |

Note: #19 appears in two rows because it is a single-file isolated CodeRabbit fix (Bucket B by file-conflict policy) but its semantics depend on the OQ-019 / admin-dashboard "what's the right scope?" question (Bucket C lens). The recommended posture: **plan now, ship the mechanical change post-Phase-6 with OQ-019 referenced as a follow-up** — see §4.

**Recommended first batch the user can hand off this week (highest confidence Bucket A):**

1. **Batch CR-MECH** — issues #20, #21, #22, #23, #25, #27, #28 (7 issues) — all single-file mechanical CodeRabbit follow-ups, zero Phase 6 file overlap, total expected effort ~half day. PR title: `chore(v1-cleanup): mechanical CR follow-ups from PR #10`.
2. **Batch CR-PROBES** — issues #26, #28 (the two `actix_smoke_probe.sh` cross-platform/AWK-state-machine fixes plus the `notify_smoke_probe.sh` zero-notify guard). All under `scratch/phase-5c-probes/`, zero crate overlap. PR title: `chore(probes): cross-platform + AWK-robustness for phase-5c smoke probes`.
3. **Batch CR-DB-INDEX** — issue #24 alone (new migration adding partial index on `surety(sponsored_id) WHERE revoked_at IS NULL`). Migration timestamp must be `> 2026-04-20-000100-0000` (latest existing) but **also strictly different from any timestamp Phase 6 task 70 will pick** — Phase 6 uses `> 2026-04-20-000000-0000` per its plan §11 task 70. Coordinate timestamp choice with advisor (use `2026-04-21-...` or later to give Phase 6 priority on `2026-04-20-...`).

**Surprises uncovered:**

- **Issue #16 (OQ-018 admin-config-write endpoint)** is the *only* issue with a hard design blocker. It needs the admin-dashboard PRD before code lands. Per the user's framing, the dashboard PRD reframes most "what's the default?" questions as "ship a sensible default + make it configurable." This **does not, however, upgrade any other v1 issue from C → A** — the 4 other Bucket-C items (#11, #14, #15, see also #19 dual-list) are not waiting on a config decision; they are waiting on jury-mechanics-v1, appeal-flow-v1, or community-membership-v1 design. See §4.
- **Issue #29** (e2e.rs config-flip test flakiness at line 2715) sits on `crates/server/tests/e2e.rs` — the same file Phase 6 task 77 adds `sanction_notice_round_trip` to. Not a hard conflict (different test functions in the same file), but **rebase risk is real** if Phase 6 lands a sweeping reorg of test helpers. Recommended: hold #29 until Phase 6 merges, then ship in a 1-line fix PR.
- **Issue #19** (community-scope `count_active_sanctions`) modifies `crates/db_views/reputation/src/impls.rs` (the `count_active_sanctions` function, line 42) and threads `Option<CommunityId>` into `crates/api/api/src/governance/get_my_reputation.rs:41`. **Phase 6 does not touch either file**, but the semantic change overlaps with the `active_sanctions` constructor question in #31, so batch them.
- **Issue #30** (governance-ai-review 413) is infrastructure — workflow YAML only. Zero code conflict. Bucket B because the user may want to wait until after Phase 6 ships before deciding among options A–E (the Phase 6 PR will be the next 413 victim).

---

## §2 Per-Issue Triage Table

Files verified by `Read` / `Glob` against current `governance-v0` HEAD (`ddb9fc4e9`).

| # | Title (short) | Bucket | Files (verified) | Phase-6 conflict | Effort | Batch | One-line rationale |
|---:|---|---|---|---|---|---|---|
| 11 | original-reporter appeals on `request_appeal` | **C** | `crates/api/api_crud/src/governance/request_appeal.rs` (line 6 doc + handler logic) | none — Phase 6 doesn't touch this file | M | jury-v1 PRD batch | Needs appeal-flow-v1 PRD to define who-can-appeal-when matrix; not a single mechanical change |
| 12 | add `assignee` filter to `list_cases` DTO | **A** | `crates/api/api_common/src/governance.rs` (DTO `ListGovernanceCases`), `crates/api/api/src/governance/list_cases.rs` (line 9 doc + handler), `crates/db_views/governance_case/src/impls.rs` (`CasesFilter`) | none | S | filter-batch | Mechanical: add `assignee: Option<PersonId>` to DTO, thread into `CasesFilter`, add Diesel filter clause. No Phase 6 overlap. (Re-bucketed from C → A: this is a pure additive DTO change, no new policy.) |
| 13 | per-community permission filter on `list_cases` | **C** | same as #12 + new permission helper | none on files; design overlap with community-membership-v1 | M | community-perm-v1 batch | Needs community-membership policy — what counts as "belongs to community"? Subscriber, mod, follower, ever-posted? Ship after a community-perm PRD or alongside #12 in a richer DTO that gates on a clear answer. |
| 14 | re-jury path for Appealed cases | **C** | `crates/api/api/src/governance/admin_close_case.rs`, `crates/api/api_crud/src/governance/request_appeal.rs`, possibly new handler `re_jury` | none | L | jury-v1 PRD batch | Needs sub-PRD: panel size for re-jury (5 vs 7?), threshold change, who picks the new panel, what happens to original jurors. Block on PRD before code. |
| 15 | formalise appeal window with bounded duration | **C** | `crates/api/api_crud/src/governance/request_appeal.rs` (lines 11–12 doc), `crates/api/api/src/governance/config.rs` (new config key `governance.appeal.window_hours`), DTO surface | none | S–M | jury-v1 PRD batch | Needs OQ-style decision (window length, instance vs community scope). Once decision lands, the code change is small. **Admin-dashboard PRD likely upgrades to A** because "ship a default + make it configurable" pattern fits perfectly — flagged in §4. |
| 16 | OQ-018 admin config-write HTTP endpoint | **C** | new files: `crates/api/api/src/governance/admin_set_config.rs`, route in `crates/api/api/src/governance/mod.rs`, DTO in `crates/api/api_common/src/governance.rs`, e2e test | none on existing files | L | dashboard-PRD batch | Hard design blocker per OQ-018 (still open). Per-key vs batch, dry-run shape, instance-only vs community-scoped admin gating, audit-trail granularity. **The admin-dashboard PRD this issue is gating on is the same PRD that unblocks #15 and shapes #19** — write the PRD first, then this issue and #15 land together. |
| 19 | community-scope `count_active_sanctions` in `get_my_reputation` | **B** *(C-flavoured)* | `crates/db_views/reputation/src/impls.rs:42` (signature change), `crates/api/api/src/governance/get_my_reputation.rs:41` (call-site update) | none on files; **but** semantic overlap with #31 + #20 (all touch reputation read-path) | S | reputation-cleanup batch | Mechanical fix is small (add `community_id: Option<CommunityId>` param, add `.filter(...)` for community-scoped sanctions). The "what *should* community-scope mean for sanctions?" question is real but answerable from existing data model — `sanction.community_id` already exists. Plan now, ship after Phase 6 to avoid touching the reputation read-path concurrently with Phase 6's reputation-event-driven federation work. |
| 20 | harden staleness_check against interval overflow + soften empty-table | **A** | `crates/api/api/src/governance/reputation_snapshot.rs:440–459` (`check_snapshot_staleness`) | none | XS | CR-MECH | Two-line fix: `interval_s.saturating_mul(2)` + downgrade `tracing::error!` to `tracing::warn!` (or empty-arm pre-check). Pure mechanical CR follow-up. |
| 21 | drop `Copy` on `ReputationBuckets` + `AdminReputationStatsResponse` | **A** | `crates/api/api_common/src/governance.rs:336` and `:339` | none | XS | CR-MECH | Single-line derive change × 2. Forward-compat hygiene. |
| 22 | bound limit on `list_capability_changed_entries_since` | **A** | `crates/db_views/governance_modlog/src/impls.rs:200–217` | none | XS | CR-MECH | Add `let limit = limit.min(MAX_LIMIT);` at top of function. One-line fix. |
| 23 | serde serialization for `governance_log` signature as base64 | **A** | `crates/db_views/governance_modlog/src/lib.rs:85–94` (struct `CapabilityChangeLogEntry`); possibly add `serde_bytes` workspace dep | **near-miss** — Phase 6 adds 4 new entry-kind const strings to `crates/api/api/src/governance/governance_log.rs`, but does NOT change the `CapabilityChangeLogEntry` view-struct. Independent. | XS | CR-MECH | Add `#[serde(with = "serde_bytes")]` on `signature: Option<Vec<u8>>` (and a tiny ts-rs annotation tweak). One-line + dep-add. |
| 24 | partial index on `surety(sponsored_id) WHERE revoked_at IS NULL` | **A** | new migration `migrations/<ts>_add_surety_active_index/{up,down}.sql` | **timestamp-coordination only** — Phase 6 task 70 picks `> 2026-04-20-000000-0000`. Use `2026-04-21-...` or later for #24's migration to give Phase 6 priority. | XS | CR-DB-INDEX | One CREATE INDEX (partial) + one DROP INDEX. Standard Postgres pattern, already named in CR. Migration filename must not collide with Phase 6's `add_federation_attestations`. |
| 25 | RCA evidence examples should use batch-file reproductions | **A** | `.claude/PRPs/debug/rca-issue-8-cargo-test-exit-code-masking.md` (docs only) | none | XS | CR-MECH (docs subset) | Markdown-only edit. Add two `.bat` reproductions (broken + fixed) to existing RCA. |
| 26 | actix_smoke_probe route threshold + cross-platform path | **A** | `scratch/phase-5c-probes/actix_smoke_probe.sh:13` | none | S | CR-PROBES | Update header comment + gate `cmd //c` behind `[[ "$OSTYPE" == "msys"* ]]`. Bash script only. |
| 27 | actix_smoke_probe AWK brittleness on lib.rs reformat | **A** | `scratch/phase-5c-probes/actix_smoke_probe.sh:35` (AWK_BLOCK pattern) | none | S | CR-PROBES | Replace fixed end-marker AWK with paren-depth state machine. Bash/awk only. |
| 28 | notify_smoke_probe should fail on zero-notify | **A** | `scratch/phase-5c-probes/notify_smoke_probe.sh:81` | none | S | CR-PROBES | Capture background `LISTEN psql` stdout/stderr to file; assert non-empty after wait; non-zero exit otherwise. |
| 29 | config-flip assertion determinism in e2e.rs | **B** | `crates/server/tests/e2e.rs:2715` | **soft conflict** — Phase 6 task 77 adds new test `sanction_notice_round_trip` to the same file (different function, but rebase risk if helper layout reorganises). | XS | post-phase-6 | One-line assertion change (or 2-line seeding adjustment). Hold until Phase 6 merges to avoid double-touching e2e.rs. |
| 30 | governance-ai-review workflow 413s on phase PRs | **B** | `.github/workflows/governance-ai-review.yml` (no code conflict) | none on workflow file; but Phase 6 PR will be the next 413 victim | M (decision) / S (impl) | post-phase-6 | Decide between options A–E from issue body. Workflow infra, no code overlap. Plan now (decision), ship after Phase 6 to gather one more data point (Phase 6 PR will fail too — strengthens case). |
| 31 | make `active_sanctions` constructor explicit (`From` impl defaults to 0) | **A** | `crates/db_views/reputation/src/lib.rs:74–89` (the `From<&ReputationSnapshot> for ReputationSummaryView` impl — note: issue body says `ReputationSummary`; actual struct is `ReputationSummaryView`) | none — Phase 6 doesn't touch reputation views | S | reputation-cleanup batch (with #19) | Pick option B (named constructor `from_snapshot_needing_sanction_count`) or C (`Option<i64>`). Touches one impl + 1–2 call sites in `crates/api/api/src/governance/get_my_reputation.rs:43–46` and `crates/db_views/reputation/src/impls.rs::build_summary_view` (line 56). |

---

## §3 Recommended Batch Groups

### Batch CR-MECH — `chore(v1-cleanup): mechanical CR follow-ups from PR #10`
**Issues:** #20, #21, #22, #23, #25
**Files:** `crates/api/api/src/governance/reputation_snapshot.rs`, `crates/api/api_common/src/governance.rs`, `crates/db_views/governance_modlog/src/{impls.rs,lib.rs}`, `.claude/PRPs/debug/rca-issue-8-cargo-test-exit-code-masking.md`
**Scope (one paragraph):** Five single-file mechanical fixes from CodeRabbit on PR #10. Each is XS effort and produces a clear cargo-check + clippy + (where relevant) e2e signal. Bundle as one commit-per-issue, one PR. Adds `serde_bytes` workspace dep (#23). No new design surface, no Phase 6 conflict, no migration. Ships in a single review pass; CodeRabbit will likely re-affirm in <30 min.
**Estimated PR size:** ~50 LOC across 4 files + 1 doc edit.
**Validation:** `cargo check --workspace --features full` + `cargo clippy -p lemmy_api -p lemmy_api_common -p lemmy_db_views_governance_modlog --no-deps -- -D warnings`. No e2e required (no behaviour change).

### Batch CR-PROBES — `chore(probes): cross-platform + AWK-robustness for phase-5c smoke probes`
**Issues:** #26, #27, #28
**Files:** `scratch/phase-5c-probes/{actix_smoke_probe.sh,notify_smoke_probe.sh}` (no Rust)
**Scope:** Three bash/awk hardening fixes. (1) Cross-platform OS detection on `cmd //c`, (2) replace fixed-marker AWK with paren-depth state machine for governance-scope extraction, (3) capture LISTEN psql output and fail loud on zero-notify. All in `scratch/` — zero impact on crates, builds, or CI Rust gates. Pure shell-script polish that strengthens the smoke probes against future code reformats.
**Estimated PR size:** ~80 LOC across 2 files.
**Validation:** Run each probe locally (sh on Windows via Git Bash and on Linux via WSL or CI runner) and assert exit-code parity vs current behaviour on the unmodified governance scope.

### Batch CR-DB-INDEX — `perf(governance): partial index on surety(sponsored_id) for active-record lookups`
**Issues:** #24
**Files:** new `migrations/2026-04-21-XXXXXX-0000_add_surety_sponsored_id_active_index/{up,down}.sql`, `crates/db_schema/src/schema.rs` (regenerated by `diesel print-schema`)
**Scope:** One new migration adding `CREATE INDEX surety_sponsored_id_active ON surety (sponsored_id, sponsor_id) WHERE revoked_at IS NULL;` and the matching DROP. Mirrors the active-record-index pattern Postgres docs recommend for nullable-timestamp columns. **Coordinate timestamp with advisor**: Phase 6 task 70's migration is `> 2026-04-20-000000-0000`; pick `2026-04-21-...` or later for #24 to ensure deterministic ordering. Cargo will regenerate `schema.rs` on `diesel print-schema`; no manual schema edits.
**Estimated PR size:** 2 SQL files + auto-generated `schema.rs` diff.
**Validation:** `cargo check -p lemmy_db_schema --features full`; verify `EXPLAIN` on `select_eligible_jurors`'s EXISTS subquery shows index-scan instead of seq-scan via a smoke probe in `scratch/`.

### Batch FILTER-DTO — `feat(governance): assignee filter on list_cases (v1)`
**Issues:** #12 (alone — #13 deferred to community-perm PRD)
**Files:** `crates/api/api_common/src/governance.rs` (DTO `ListGovernanceCases` + ts-rs export), `crates/api/api/src/governance/list_cases.rs` (handler doc + thread param), `crates/db_views/governance_case/src/impls.rs` (`CasesFilter` + Diesel filter clause)
**Scope:** Pure additive DTO change. Add `assignee: Option<PersonId>` to `ListGovernanceCases`; thread to `CasesFilter`; add `.filter(jury_assignment::juror_id.eq(...))` join clause when `assignee = Some(_)`. No new policy — assignee-by-PersonId is well-defined, no community-scope ambiguity (that's #13's territory). Update handler doc-comment to remove "v1 scope per plan §11.7 GOTCHA" line for the assignee field (#13's per-community filter line stays).
**Estimated PR size:** ~30 LOC + e2e test extension.
**Validation:** Add a sub-test to existing `list_cases` e2e or write a new small test `list_cases_filters_by_assignee`.

### Batch REP-CLEANUP — `refactor(reputation): explicit From + community-scoped sanction count`
**Issues:** #19 + #31 (sequenced one commit each)
**Files:** `crates/db_views/reputation/src/lib.rs` (drop or rename `From` impl), `crates/db_views/reputation/src/impls.rs:42` (`count_active_sanctions` signature `Option<CommunityId>`), `crates/api/api/src/governance/get_my_reputation.rs:41–46` (call-site update + remove `..ReputationSummaryView::from(&snapshot)` shorthand)
**Scope:** Two related changes that touch the same reputation read-path. (1) #31 drops the silent-default `From<&ReputationSnapshot>` pattern in favour of either named-constructor or `Option<i64>` field shape — this forces every call site to think about `active_sanctions` explicitly. (2) #19 then threads `Option<CommunityId>` through `count_active_sanctions` so community-scoped reputation reads return community-scoped sanction counts (eliminating the misleading composite where snapshot is community-scoped but count is instance-wide). Recommended sequencing: ship #31 first (mechanical refactor), then #19 (semantic fix that benefits from the explicit constructor). **Hold until after Phase 6 merges** to avoid touching the reputation read-path concurrently with Phase 6 task 76's federation publish (which fires *from* `submit_jury_vote` — separate call path but adjacent in the codebase).
**Estimated PR size:** ~60 LOC + e2e test for community-scoped count.

### Batch INFRA-AI-413 — `infra(ci): pick mitigation for governance-ai-review 413 on phase PRs`
**Issues:** #30 (alone)
**Files:** `.github/workflows/governance-ai-review.yml` (option C: rewrite to sample N most-interesting files; option D: delete file + update `.coderabbit.yaml` and README)
**Scope:** Decision-then-impl. The decision is "which of options A–E in the issue body do we pick?" — the user should choose. Once chosen, the workflow change is small (~50 LOC for option C, or 5 LOC + readme blurb for option D). **Hold until after Phase 6 merges** because the Phase 6 PR will be the next 413 victim and provides a fresh data point. CodeRabbit Pro is the authoritative review path either way.

### Issues NOT batched — `Watch list / PRD-blocked`
**Issues:** #11, #13, #14, #15, #16, #29
- #11, #14: jury-mechanics-v1 PRD (re-jury path, who-can-appeal matrix)
- #13: community-membership-v1 PRD
- #15: admin-dashboard PRD upgrade candidate (see §4)
- #16: admin-dashboard PRD blocking (the canonical OQ-018 issue)
- #29: hold one PR cycle for Phase 6 to land first

---

## §4 Watch-List — Bucket Assignment Depends on a Phase-6 Choice or v1 PRD

### W1 — Issue #29 (e2e.rs config-flip flakiness)
**Status:** Bucket **B** today; could become A if Phase 6 task 77 lands cleanly without restructuring `e2e.rs` test helpers.
**Phase-6 dependency:** task 77 adds `sanction_notice_round_trip` test function and may introduce 2-DB test scaffolding that other tests adopt. If the scaffolding refactor relocates `seed_case` / `eligibles[]` helpers around line 2715, #29's fix needs to be re-anchored.
**Decision rule:** Wait for Phase 6 PR to open; check whether `crates/server/tests/e2e.rs` got reorganised. If yes, hold #29 until merge. If no (Phase 6 only appends a new test function), upgrade #29 to A and ship in the next mechanical batch.

### W2 — Issue #15 (formalise appeal window) + admin-dashboard PRD
**Status:** Bucket **C** today (waits on a "what's the default appeal window?" answer).
**Upgrade path:** **The admin-dashboard PRD reframes "what's the default?" as "ship a sensible default (say, 7 days) + make it a configurable governance_config key + UI to tune."** If the user adopts that framing, #15 upgrades to **B** — the implementation becomes mechanical (add config key, read it in `request_appeal.rs` to reject post-window appeals, surface in DTO). The remaining decision shrinks to "what's the v0 default value?" which is one judgment call, not a multi-week design.
**Decision rule:** When the admin-dashboard PRD is written, re-triage #15 immediately. Likely outcome: B → ships alongside #16 once #16's HTTP endpoint exists.

### W3 — Issue #19 (community-scope count_active_sanctions)
**Status:** Bucket **B** today. Single-file mechanical fix, no Phase 6 conflict, but bundled with #31 in REP-CLEANUP for cohesion.
**Upgrade-to-A blocker:** none structural — purely a "ship after Phase 6 merges to avoid concurrent reputation-path edits" precaution. Could be A if the user is comfortable with two PRs touching reputation code in flight simultaneously (Phase 6 task 76 modifies `submit_jury_vote.rs` which writes reputation events; #19 modifies the read path — they don't overlap, so risk is small).

### W4 — Issue #16 (OQ-018 admin config-write endpoint)
**Status:** Bucket **C**, blocking on admin-dashboard PRD (the canonical instance of OQ-018 deferral).
**Knock-on effects:** Once the admin-dashboard PRD lands, #16 upgrades to a single-PR implementation effort (~L). The PRD's per-key vs batch / dry-run / instance-vs-community decisions are all in OQ-018's "Current lean" — that lean could become the PRD verbatim, in which case #16 is implementable in 1–2 days post-PRD.

### W5 — Issues #11, #14 (jury-mechanics-v1 PRD)
**Status:** Bucket **C**, blocking on a sub-PRD covering: who can appeal (target-only vs original-reporter vs both with separate flows), re-jury panel size (5 vs 7), threshold change for re-jury, original-juror exclusion. Both issues are moderate effort (M) once the PRD lands.
**Combined PR potential:** #11 + #14 + #15 (if #15 hasn't been upgraded by then) could ship as one "appeal-flow-v1" feature PR. Estimate: 1 week post-PRD.

---

## §5 Cross-Reference — Bucket A/B Issues + `gh` Commands and File Paths

For a planning agent picking up any Bucket A or B issue:

### Issue #20 — staleness_check hardening (Bucket A, XS)
```
gh issue view 20 --repo barrie-cork/lemmy --json number,title,body
```
- `crates/api/api/src/governance/reputation_snapshot.rs:438-459`

### Issue #21 — drop Copy on reputation aggregates (Bucket A, XS)
```
gh issue view 21 --repo barrie-cork/lemmy --json number,title,body
```
- `crates/api/api_common/src/governance.rs:336` (ReputationBuckets derive)
- `crates/api/api_common/src/governance.rs:339` (AdminReputationStatsResponse derive)

### Issue #22 — bound limit on capability_changed listing (Bucket A, XS)
```
gh issue view 22 --repo barrie-cork/lemmy --json number,title,body
```
- `crates/db_views/governance_modlog/src/impls.rs:198-217` (`list_capability_changed_entries_since`)

### Issue #23 — base64 signature serialization (Bucket A, XS)
```
gh issue view 23 --repo barrie-cork/lemmy --json number,title,body
```
- `crates/db_views/governance_modlog/src/lib.rs:85-94` (`CapabilityChangeLogEntry`)
- `crates/db_views/governance_modlog/Cargo.toml` (add `serde_bytes` if not in workspace deps)

### Issue #24 — partial index on surety active records (Bucket A, XS)
```
gh issue view 24 --repo barrie-cork/lemmy --json number,title,body
```
- new: `migrations/2026-04-21-NNNNNN-0000_add_surety_sponsored_id_active_index/{up,down}.sql`
- regenerated: `crates/db_schema/src/schema.rs`
- read-only refs: `crates/api/api/src/governance/jury_common.rs:42` (the EXISTS query that benefits)

### Issue #25 — RCA batch-file reproductions (Bucket A, XS, docs only)
```
gh issue view 25 --repo barrie-cork/lemmy --json number,title,body
```
- `.claude/PRPs/debug/rca-issue-8-cargo-test-exit-code-masking.md` (append two `.bat` repros to evidence section near line 49)

### Issue #26 — actix_smoke_probe cross-platform (Bucket A, S)
```
gh issue view 26 --repo barrie-cork/lemmy --json number,title,body
```
- `scratch/phase-5c-probes/actix_smoke_probe.sh:13` (header + cargo-check OS gate)

### Issue #27 — actix_smoke_probe AWK state machine (Bucket A, S)
```
gh issue view 27 --repo barrie-cork/lemmy --json number,title,body
```
- `scratch/phase-5c-probes/actix_smoke_probe.sh:35` (AWK_BLOCK)

### Issue #28 — notify_smoke_probe zero-notify guard (Bucket A, S)
```
gh issue view 28 --repo barrie-cork/lemmy --json number,title,body
```
- `scratch/phase-5c-probes/notify_smoke_probe.sh:81` (background LISTEN handling)

### Issue #31 — explicit active_sanctions constructor (Bucket A, S)
```
gh issue view 31 --repo barrie-cork/lemmy --json number,title,body
```
- `crates/db_views/reputation/src/lib.rs:74-89` (the `From` impl for `ReputationSummaryView`)
- `crates/db_views/reputation/src/impls.rs:55-` (`build_summary_view` — alternate constructor pattern already in place)
- `crates/api/api/src/governance/get_my_reputation.rs:43-46` (call site using `..ReputationSummaryView::from(&snapshot)`)

### Issue #12 — assignee filter (Bucket A, S, batched alone or with #19 mass v1-cleanup if user prefers)
```
gh issue view 12 --repo barrie-cork/lemmy --json number,title,body
```
- `crates/api/api_common/src/governance.rs` (`ListGovernanceCases` DTO)
- `crates/api/api/src/governance/list_cases.rs:9` (doc + handler)
- `crates/db_views/governance_case/src/impls.rs` (`CasesFilter` + `list_cases_filtered`)

### Issue #19 — community-scope count_active_sanctions (Bucket B, S)
```
gh issue view 19 --repo barrie-cork/lemmy --json number,title,body
```
- `crates/db_views/reputation/src/impls.rs:42-53` (`count_active_sanctions` — add `Option<CommunityId>` param)
- `crates/api/api/src/governance/get_my_reputation.rs:41` (call site — pass `data.community_id`)
- read-only ref for FK column existence: `migrations/2026-04-15-100100-0000_add_governance_core/up.sql` (sanction.community_id)

### Issue #29 — config-flip determinism (Bucket B, XS, hold for Phase 6 merge)
```
gh issue view 29 --repo barrie-cork/lemmy --json number,title,body
```
- `crates/server/tests/e2e.rs:2705-2730` (BRANCH 3 of the `assignment_cap_branches` test)

### Issue #30 — governance-ai-review 413 mitigation (Bucket B, M decision / S impl)
```
gh issue view 30 --repo barrie-cork/lemmy --json number,title,body
```
- `.github/workflows/governance-ai-review.yml`
- referenced: `.coderabbit.yaml` (if option D — disable + update CR config)

---

## Appendix — Phase 6 modified-file inventory (cross-reference source)

For verification, Phase 6's MODIFIED FILES list (plan §"Files to Change"):

- `crates/db_schema/src/schema.rs`
- `crates/db_schema/src/source/governance/mod.rs`
- `crates/db_schema/src/newtypes.rs`
- `crates/apub/objects/src/lib.rs`
- `crates/apub/activities/src/lib.rs`
- `crates/apub/activities/src/activity_lists.rs`
- `crates/apub/apub/src/lib.rs`
- `crates/api/api/src/governance/governance_log.rs` (adds 4 new entry-kind const strings)
- `crates/api/api/src/governance/submit_jury_vote.rs` (insert federation publish call between lines 395 and 397)
- `crates/server/tests/e2e.rs` (new `sanction_notice_round_trip` test)
- `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md`
- `Cargo.lock`

**Cross-check vs v1 issues:**
- `governance_log.rs` — issues #22, #23 touch the *modlog view* (`governance_modlog/src/{impls,lib}.rs`), not this file. No conflict.
- `submit_jury_vote.rs` — no v1 issue touches it. No conflict.
- `e2e.rs` — issue #29 touches it (line 2715); soft-conflict / rebase risk. Hold #29.
- `schema.rs` / `newtypes.rs` / `governance/mod.rs` — issue #24 regenerates `schema.rs` via `diesel print-schema` (new index). Trivial merge if Phase 6 lands first; pick a later timestamp than Phase 6 to be safe.
- `activity_lists.rs`, `lib.rs` (apub) — no v1 issue touches AP code.
- `SUBSCRIPTIONS.md`, `Cargo.lock` — no v1 issue touches them.

**Net result:** zero hard Phase 6 conflicts in any of the 19 v1 issues. All conflicts are either (a) soft `e2e.rs` co-occupancy (#29) or (b) timestamp coordination on a new migration (#24). The Bucket A set is genuinely shippable in parallel with Phase 6.
