---
phase: v1-AD
sub_phases: [v1-AD-a, v1-AD-b, v1-AD-c, v1-AD-d]
first_plan_pr_opened: 2026-04-19T12:04:22Z   # PR #44 (AD-a plan)
last_impl_pr_merged:  2026-04-23T16:51:31Z   # PR #87 (AD-d impl)
wall_clock_elapsed:   ~4d 5h
author: Barrie (solo) + Claude Opus 4.7 (1M ctx)
---

# v1-AD meta-retro — admin-dashboard PRD consolidated across AD-a..AD-d

v1-AD was the first v1 PRD to land. Four sub-phases (AD-a foundation,
AD-b config HTTP endpoints, AD-c rule-sets + case-open pin, AD-d
dashboard aggregate + SSE audit stream), four implementation PRs, four
plan PRs, and one OQ-resolution PR. This document consolidates what
each sub-phase contributed, how the process evolved across them, and
what the macro-retro says that the per-sub-phase retros don't.

---

## 1. Timeline — PR-level

All timestamps are ISO 8601 UTC, taken from `gh pr view --json`.

| PR  | Title                                                                       | Opened                | Merged                | Open→merge  | +add   | -del | Commits |
|-----|-----------------------------------------------------------------------------|-----------------------|-----------------------|-------------|--------|------|---------|
| #44 | docs(plan): v1-AD-a — admin-dashboard sub-phase A                           | 2026-04-19 12:04:22Z  | 2026-04-19 19:47:55Z  | 7h 43m      | 6 161  | 0    | 8       |
| #72 | v1-AD-a — admin-dashboard foundation (config metadata registry + schema)    | 2026-04-20 07:27:20Z  | 2026-04-20 08:05:47Z  | 38m         | 1 713  | 4    | 13      |
| #74 | docs(oq-v1-ad): resolve OQ-V1-AD-01/02/03 + reshape PRD §4.3                | 2026-04-20 11:55:35Z  | 2026-04-20 12:41:12Z  | 46m         | 31     | 13   | —       |
| #75 | docs(v1-AD-b): plan — admin config HTTP endpoints (write/read/audit)        | 2026-04-20 12:24:18Z  | 2026-04-20 12:42:01Z  | 18m         | 1 467  | 14   | 5       |
| #76 | Phase v1-AD-b — Admin Dashboard config HTTP endpoints (write/read/audit)    | 2026-04-20 18:08:01Z  | 2026-04-20 23:10:55Z  | 5h 3m       | 2 739  | 8    | 14      |
| #81 | v1-AD-c — rule-sets + case-open pin + #77/#78 carry-forward                 | 2026-04-21 19:03:33Z  | 2026-04-22 16:19:24Z  | 21h 16m     | 4 148  | 134  | 15      |
| #86 | docs(plan): v1-AD-d dashboard aggregate + SSE audit stream                  | 2026-04-22 20:39:58Z  | 2026-04-22 21:39:53Z  | 1h          | 2 968  | 0    | 5       |
| #87 | Phase v1-AD-d — Dashboard aggregate + SSE audit stream                      | 2026-04-23 08:56:48Z  | 2026-04-23 16:51:31Z  | 7h 55m      | 4 830  | 72   | 18      |

**Total wall-clock from first plan PR opened (2026-04-19 12:04:22Z) to
last impl PR merged (2026-04-23 16:51:31Z):** 4 days 4 hours 47 minutes.

**Implementation diff total across the four impl PRs (#72 + #76 + #81
+ #87):** +13 430 / −218 across 60 commits. Plan PRs add +10 627 more
in design documents (#44 + #75 + #86 + the OQ-reshape #74).

### 1.1 Sub-phase characterisation

| Sub-phase | PR (impl) | Scope in one sentence                                                               | Impl wall-clock (open→merge) | Notable |
|-----------|-----------|-------------------------------------------------------------------------------------|------------------------------|---------|
| AD-a      | #72       | 27-key config metadata registry + 2 `ENTRY_KIND_*` consts + 4 schema additions      | 38m                          | Clean ralph loop; one task-5 correction commit |
| AD-b      | #76       | `admin_set_config` / `admin_get_config` / `admin_get_config_audit` HTTP endpoints    | 5h 3m                        | First CR round on an AD PR (5 mechanical findings) |
| AD-c      | #81       | `admin_create_rule_set` + `admin_list_rule_sets`, `moderation_case` snapshot pin     | 21h 16m (overnight)          | 3 CR review rounds + #77/#78 carry-forward |
| AD-d      | #87       | `admin_dashboard` aggregate + `admin_audit_stream` SSE                               | 7h 55m                       | 4 CR review rounds (26 findings, 10 fix-in-PR) |

---

## 2. What each sub-phase added to the production tree

Derived from `git log` on `governance-v0` restricted to handler files
under `crates/api/api/src/governance/`.

### AD-a foundation (PR #72, merged 2026-04-20 08:05:47Z)
- New consts: `ENTRY_KIND_ADMIN_CONFIG_CHANGED`, `ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED`
- New schema columns: `moderation_case.applied_config_snapshot`, `moderation_case.rule_set_version_id`
- `CONFIG_KEY_METADATA` array (61 rows) + `SEEDED_KEYS_WITH_CONSTS`
- Diesel models for `rule_set_version`, `sponsor_allowlist` (no handlers yet)
- 27 seeded config rows in `governance_config` (via migration)

### AD-b config HTTP endpoints (PR #76, merged 2026-04-20 23:10:55Z)
- 3 new handlers: `admin_set_config` (POST), `admin_get_config` (GET), `admin_get_config_audit` (GET)
- 9 DTOs in `api_common/governance.rs`
- Wire-format parsing via `Scope::parse_wire`, `ConfigValueType::parse_wire`
- `process_set_config` with dry-run impact dispatcher (scaffolded in task 3)
- Payload-parity unit tests between shell wrapper and HTTP handler
- First CR round on AD work: 5 mechanical findings, 1 commit

### AD-c rule-sets + snapshot pin (PR #81, merged 2026-04-22 16:19:24Z)
- 2 new handlers: `admin_create_rule_set` (POST), `admin_list_rule_sets` (GET)
- Issue #78 fix: `Scope::parse_wire` rejects non-positive `community_id` with typed `ScopeParseError`
- Issue #77 fix: `admin_config_changed` payload carries `previous_value` + `previous_from`
- `case_open_snapshot` helper: pins all 7 `requires_re_jury` keys on new-case open
- 1 new `ENTRY_KIND_RULE_SET_VERSION_CREATED` const
- `map_rsv_unique_violation` helper extracted for deterministic duplicate-version test
- `rule_text` scrubbed via `crate::governance::redaction::scrub` on list response (ADR-015)
- 8 e2e tests + 3 CR rounds (rounds 1/2 on commits `5c04e6ec2`/`6c8fc3211`, round 3 on `6baabfd7a`)

### AD-d dashboard + SSE (PR #87, merged 2026-04-23 16:51:31Z)
- 2 new handlers: `admin_dashboard` (GET aggregate), `admin_audit_stream` (GET SSE)
- Shared `audit_projection::project_to_audit_entry` extracted from AD-b
- Bounded tokio `mpsc::channel(256)` drop-policy for slow SSE clients
- Per-admin SSE-connection cap via `OnceLock<Mutex<HashSet<PersonId>>>` (409 Conflict on second)
- `governance_log_notify_trigger` migration `2026-04-20-000100-0000` (fires on `AFTER UPDATE OF signature`)
- 5 original e2e tests + 1 post-review substitution test = 6 e2e total
- 4 CR rounds (poll #3/#4/#5/#7); 26 findings total (10 fix-in-PR / 5 rebut / 11 carry-forward)

---

## 3. Process evolution across the four sub-phases

### 3.1 CR-review load doubled every sub-phase
- AD-a: 0 rounds (no CR findings on impl PR — CR caught nothing the parity test hadn't already)
- AD-b: 1 round, 5 mechanical findings
- AD-c: 3 rounds, ~8 findings across rounds (4 Major, rebuttals + fixes)
- AD-d: 4 rounds, 26 findings (triages #1..#4)

This isn't implementer drift — it's surface-area growth. AD-a was pure
schema + metadata. AD-d had two new HTTP handlers, one of them a novel
streaming pattern (hand-rolled SSE + `tokio_postgres::LISTEN`), plus
six e2e tests exercising admin-auth + NOTIFY + subprocess-level
streaming. More places to get wrong. The **fix-in-PR discipline** held
— zero regressions merged.

### 3.2 Plan PR → impl PR same-day cadence
Three of the four AD sub-phases had plan-PR-merged and impl-PR-merged
on the **same UTC day**:
- AD-a: plan #44 merged 04-19 19:47Z → impl #72 merged 04-20 08:05Z (next-day impl, 12h 18m gap)
- AD-b: plan #75 merged 04-20 12:42Z → impl #76 merged 04-20 23:10Z (same-day, 10h 28m gap)
- AD-c: plan #79 n/a — AD-c used `docs(plan)` commits on the impl branch itself (drift from the phase-branch convention; see §5.2)
- AD-d: plan #86 merged 04-22 21:39Z → impl #87 merged 04-23 16:51Z (next-day, 19h 12m gap)

Plan-then-implement with a ≤1-day gap kept plan freshness high and
reduced "what did the advisor mean here" diagnosis. By contrast,
plans that sat unimplemented (e.g. the deferred v1-JM plan) accumulate
drift.

### 3.3 Branch-manager agent landed between AD-c and AD-d
The 9 `/bm-*` commands + `branch-manager` subagent (commit
`bf5eb1a3b`, merged just before AD-d impl) replaced ad-hoc `gh pr
create` / `gh pr merge` / CR-triage work with a four-bucket discipline
(`fix-in-pr` / `rebut` / `carry-forward` / `wont-fix`) and a gitignored
findings YAML. AD-d PR #87 was the first end-to-end exercise: 4
triages, 26 findings bucketed, 11 deferred to issue #88. The separation
of impl-session from BM-session kept the impl session focused on Rust
while BM handled git topology + CR ingestion.

**Unanticipated cost:** the BM tooling itself drew CR findings (cr-1..
cr-11 on PR #87 — 11 findings against `.claude/commands/bm/*.md` and
`.claude/rules/branch-manager.md`). Consolidated into issue #88 as
carry-forward so AD-d didn't balloon. Process-level tooling under
review is a new scope category the prior bm/rules/ files had never
received.

### 3.4 pre-phase-harness-audit.md added Probe 4 (exit-code propagation)
Added mid-AD-c after the `cargo-test.bat` wrapper silently masked a
real failure (`e7cad24fd`, RCA in
`.claude/PRPs/debug/rca-issue-8-cargo-test-exit-code-masking.md`). The
four-probe audit (`-p` scoping, `--features` activation, test-target,
negative-feature) now gates every phase-start. AD-d ran all four green
in Task 0 audit (`.claude/PRPs/reports/v1-AD-d-task0-audit.md`).

### 3.5 Fix-session chain pattern emerged in AD-d
AD-d PR #87 shipped 7 CR-fix commits after the initial 11-commit impl.
Chain shape:

```
task 1-6 (11 commits)    →  push + /bm-poll-cr        → 4 fix-in-PR findings
├─ 6c1654cb7 (cr-13,18)  →  push + /bm-poll-cr        → 2 new fix-in-PR
├─ 0699a1ac0 (cr-14,15)  →  push + /bm-poll-cr + triage→ 0 new findings
├─ 546348236 (clippy)    →  push + /bm-poll-cr        → 2 new fix-in-PR (cr-21,22)
├─ 8b99a3401 (cr-21,22)  →  push + /bm-poll-cr        → 2 new fix-in-PR (cr-23,24)
├─ 7ea0844cd (cr-23,24)  →  push + /bm-poll-cr        → 1 new fix-in-PR (cr-25)
├─ 357dca6d4 (cr-25)     →  push + /bm-poll-cr        → 1 new fix-in-PR (cr-26 Critical)
└─ 83e0dfdb7 (cr-26)     →  push + /bm-poll-cr        → silent; triage + merge
```

Total: 18 commits, 8 CR poll cycles, 4 triage rounds. The fix-session
took ~2.5h of wall-clock (commit timestamps 11:34Z..16:20Z on 04-23),
all in auto mode with AskUserQuestion gates only on outbound-visible
actions (digest-post + merge). **Economy of this pattern depends on CR
having a rough 5-10 min re-review cadence** — sleeping 270s between
polls fit that; waiting longer would have slowed the chain.

---

## 4. What worked across all four sub-phases — keep

### 4.1 Plan §10 pattern-block discipline
Every sub-phase plan named exact file + line-number mirrors for new
code. AD-d's SSE skeleton cited
`crates/server/tests/e2e.rs:2979-2998` as the NOTIFY-probe mirror; the
final handler loop (`admin_audit_stream.rs:123-142`) was byte-close to
that. Plans that *don't* pin a line number (seen in drafts of early
v1-AD-a sections) cost ~5 minutes of re-grep each time.

### 4.2 Parity tests converted error-prone edits into compile-time signals
AD-a's `every_seeded_key_has_metadata` test caught the 27-row vs
61-metadata drift instantly. AD-d inherited the pattern for
`ADMIN_DASHBOARD_WIDGET_MATRIX` (plan §6.2) — widget count is asserted
at test time so a PRD update without an impl update fails fast. This
converts manual cross-referencing (easy to miss) into test-time signal
(impossible to miss).

### 4.3 Atomic task-per-commit with `feat(<scope>): task N — <summary>`
All four sub-phases held task-per-commit discipline. CR can read each
task's delta in isolation; retros can reconstruct from `git log`. Fix
commits appended as additional commits (never amended) and counted
toward the PR total. **Never squash on merge** — this is `phase-branch.md`
§"Do not squash"; AD-a-d all merged with `--merge`.

### 4.4 Captured cargo output → file, exit code preserved
Zero exit-code masking across four sub-phases. The `cargo-output-capture.md`
rule + `same-shell $?` + `no-cargo-output-paste.md` trio held. AD-c's
wrapper exit-code-masking RCA was a harness bug, not a rule failure.

### 4.5 GOTCHA sections that predict specific failure modes
AD-d's task 5 GOTCHAs (SSE `\n\n` ending, `SseGuard::Drop` abort,
`OnceLock` vs `once_cell`, `NoTls` default) all landed correctly
without re-learning. Vague "be careful" GOTCHAs don't prevent
anything — specific-failure-mode GOTCHAs do.

---

## 5. What caused friction across sub-phases — fix in future PRDs

### 5.1 Plan drift on clippy DoD (AD-a → AD-b → AD-c)
Three sub-phases drifted the clippy DoD command:
- AD-a used `-p lemmy_api --features full` (worked because handlers
  were all in `lemmy_api`)
- AD-b hit the "features full is workspace-only" trap
  (`feedback_features_full_workspace_only.md`); narrowed to
  `--workspace --features full --no-deps` mid-phase
- AD-c pre-narrowed via plan §15 Level 4 (`0dd3840e1`) — first sub-phase
  to get it right at plan-time

**Fix:** make `cargo clippy --workspace --features full --no-deps -- -D
warnings` the default for plan DoD templates. It's the only form that
works with feature propagation.

### 5.2 AD-c had no separate plan PR
AD-c's plan (`v1-admin-dashboard-c.plan.md`) landed on the impl branch
itself via `docs(plan)` commits rather than in its own PR. Reviewers
had to diff the plan against its own impl in one PR — review surface
~4 000 lines rather than 2 000 + 2 000. The plan-PR-separate convention
from AD-a/b/d should be standardised.

### 5.3 CR tooling findings spilled onto feature PRs
11 of AD-d PR #87's 26 findings targeted `.claude/commands/bm/*.md` and
`.claude/rules/branch-manager.md` — the BM tooling introduced just
before AD-d. Putting tooling-scope work on a feature-scope PR doubled
the review surface. **Fix:** tooling changes land in a dedicated
`chore(bm)` PR before the first feature-PR using the tooling.

### 5.4 `LemmyErrorType::Unknown` flattens to HTTP 400
AD-d plan §10 SSE skeleton used `actix_web::error::ErrorConflict(...)`
which, wrapped through `LemmyErrorType::Unknown`, maps to HTTP 400 —
not 409. Caught via self-review before test-writing. DQ #43's Phase
4.1.1 "HTTP Status Code Audit" step patches this at the command
template level (already applied via PR #89). **Keep the audit step in
plan DoDs.**

### 5.5 Docker daemon assumed running
AD-d session lost ~3 minutes diagnosing a testcontainers error that
read like "Postgres failed to start" but was actually "Docker daemon
not running". DQ #44 added Probe 0 (Docker preflight) to both
`/prp-core:prp-implement` Phase 4.2.0 and
`.claude/rules/pre-phase-harness-audit.md`. **Applied via PR #89.**

---

## 6. What the per-sub-phase retros DON'T say

### 6.1 Total e2e test count in tree grew from ~58 to ~90
AD-a added 2 parity tests; AD-b added ~8 config-endpoint tests; AD-c
added 8 rule-set + snapshot tests; AD-d added 6 dashboard + SSE tests.
**~24 new e2e tests** across v1-AD. Cold e2e suite runtime on Windows
grew from ~13 min (post-Phase-6 baseline) to ~16 min by end of AD-d.
The cold-build gate per sub-phase ran ~4× during v1-AD — budgeted time
well.

### 6.2 Two carry-forward issues (#77, #78) opened during AD-b closed in AD-c
Issue #77 (previous_value provenance in audit payload) and #78
(non-positive community_id parse) were opened as AD-b CR findings
deferred to AD-c scope. Both closed in AD-c commits `d623bcff5` and
`c8c29c973`. This is the carry-forward pattern working as designed —
real issues that don't belong in the current PR get tracked but not
dropped.

### 6.3 OQ-V1-AD-01/02/03 resolved via PR #74
Before AD-b impl, three OQs from the v1-AD PRD were closed via PR #74
(resolving: audit-projection shape, widget-matrix authoritative
source, LIMIT 100 disclosure pattern). **OQ resolution before
implementation prevents mid-impl PRD drift** — every time AD-d rebutted
a CR finding (cr-12, cr-16, cr-17), the rebuttal cited a PRD section
stabilised by #74. Without #74 those rebuttals would have been weaker.

### 6.4 The `v1-admin-dashboard.prd.md` itself evolved during implementation
- PR #74: OQ-reshape + §4.3 rewrite (−13 / +31 lines)
- AD-b impl (PR #76 commit `403b01b72`): §8.4 NOT5 reframing
- AD-c round-2 (PR #81 commit `6c8fc3211`): §8.4 condition 3 updated
  from "field-for-field parity" to "continuity"
- AD-d plan (PR #86): §6.2 widget-matrix pinning

**Pattern:** PRDs are reference documents but aren't frozen.
Implementation finds edge cases (e.g. `previous_value` field extending
the shell-parity contract) that require PRD amendments. The amendments
land as `docs(prd): ...` commits in the impl PR, not in separate
documentation PRs. This keeps PRD and code synchronised; the PRD never
lies about what ships.

### 6.5 Total DQ entries filed and resolved during v1-AD
From `git log` + DQ JSON:
- AD-a: DQ #36–37 (1 technical substitution, 1 attribution-correction)
- AD-b: DQ #38 (plan drift on clippy scope)
- AD-c: DQ #39–40 (rule-set scope + prp-implement patch)
- AD-d: DQ #41–46 (6 items; #42–#46 are the prp-implement hardening)

All 11 v1-AD-era DQs were resolved before or at merge. No DQ survived
a sub-phase transition. DQ #45 (test-substitution policy) was
particularly instructive: the retro-sourced question had a clear
lean (`(b)` — codify as checklist); advisor confirmed in
`e7a2ba85c`.

---

## 7. Metrics at v1-AD close (2026-04-23 16:51Z)

| Metric                                | Value      | Notes |
|---------------------------------------|------------|-------|
| AD sub-phases shipped                 | 4 / 4      | All merged into `governance-v0` |
| New production handlers               | 7          | `admin_set_config`, `admin_get_config`, `admin_get_config_audit`, `admin_create_rule_set`, `admin_list_rule_sets`, `admin_dashboard`, `admin_audit_stream` |
| New `ENTRY_KIND_*` consts             | 3          | `ADMIN_CONFIG_CHANGED`, `ADMIN_CONFIG_CHANGE_DENIED`, `RULE_SET_VERSION_CREATED` |
| New schema objects                    | 2 columns + 2 tables + 1 view | `applied_config_snapshot`, `rule_set_version_id` cols; `rule_set_version`, `sponsor_allowlist` tables; `governance_config_current` view |
| New DTOs in `api_common/governance.rs`| ~18        | Includes `AdminDashboardResponse` and all sub-widget response types |
| New e2e tests                         | ~24        | Across config, rule-sets, snapshot pin, dashboard, SSE |
| CR findings ingested                  | 40+        | Across PR #76 (5) + #81 (13) + #87 (26) |
| CR findings merged as rebuttals       | 6          | All with plan/PRD/ADR citations |
| CR findings deferred to issues        | 13+        | #77, #78, #82–#85, #88 (consolidated) |
| ADR violations                        | 0          | None detected or reported |
| Regressions merged                    | 0          | No post-merge `fix(v1-AD-<x>)` reverts |

---

## 8. Recommendations for v1-JM (next PRD)

1. **Plan PR separate from impl PR.** Follow AD-a/b/d convention, not AD-c.
2. **Pre-resolve OQs before impl, as in PR #74.** If v1-JM PRD has open OQs, file a resolution PR first.
3. **BM tooling is now usable from task 1.** v1-JM should use `/bm-cut` → `/bm-push` → `/bm-pr` → `/bm-poll-cr` → `/bm-triage` → `/bm-merge` from the start. No more ad-hoc `gh pr create`.
4. **Expect ~20–30 CR findings for a 5-task sub-phase.** Budget 1–2h of fix-session wall-clock per review round. Four rounds is not unusual.
5. **Parity tests for any new metadata array.** Every ~N-key metadata needs an N-matching-M assertion; copy the AD-a `every_seeded_key_has_metadata` pattern.
6. **Status-code audit in Task 0 when plan cites specific 4xx.** Per DQ #43 applied in PR #89. Direct `HttpResponse::<Status>()` or explicit `LemmyErrorType::status_code()` special-case; `ErrorConflict` et al. flatten to 400.
7. **Docker daemon preflight (Probe 0) before first e2e.** Already applied to `pre-phase-harness-audit.md` and `/prp-core:prp-implement`.
8. **Task-resume detection in Phase 1.4** of `/prp-core:prp-implement`. Picks up from `git log`; prevents silent task-1 re-do on session resume. **Applied via PR #89.**
9. **Follow-up GH issues for v1 limitations in Phase 5 REPORT.** Implementer drafts the `gh issue create` invocations; advisor files. **Applied via PR #89.**

---

## 9. Closing — v1-AD shipped, v1-JM next

All four v1-AD sub-phases merged. PR #87 (AD-d) is the last. Issue #88
tracks BM-tooling carry-forward. MEMORY.md needs updating to reflect
v1-AD CLOSED (from v1-AD-c CLOSED currently).

**Next PRD in sequence per `project_v1_prd_sequence.md`:** v1-JM (jury
mechanics). The v1-JM PRD file at
`.claude/PRPs/prds/v1-jury-mechanics.prd.md` is already drafted. Plan
PR cutting is the next action.
