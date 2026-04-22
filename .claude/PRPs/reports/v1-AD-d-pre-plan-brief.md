# v1-AD-d pre-plan brief

**Written:** 2026-04-22 by advisor at v1-AD-c phase close
**Purpose:** Self-contained context for a fresh CC session to run `/prp-plan` targeting v1-AD-d. Read this, then run `/prp-plan "v1-admin-dashboard-d — dashboard aggregate + SSE audit stream"`.

## Scope of v1-AD-d (from v1-AD-b plan §6)

**Two endpoints:**
1. `GET /admin/dashboard` — aggregate reader for all config + audit state (feeds the server-rendered HTML page path, but HTML itself is deferred to v1-AD-e per OQ-V1-AD-01 resolution)
2. `GET /admin/audit/stream` — SSE endpoint streaming `admin_config_changed` + `admin_config_change_denied` events hand-rolled via `async-stream` (per OQ-V1-AD-02 resolution — no additional dep)

**What v1-AD-d does NOT ship:**
- Askama HTML pages (OQ-V1-AD-01 resolved "defer to v1.x / v1-AD-e")
- SSE via tower-sse or another crate (OQ-V1-AD-02 resolved "hand-roll with async-stream")

## Pre-plan readiness — ALL GREEN

| Gate | State |
|---|---|
| PRD exists | ✅ `.claude/PRPs/prds/v1-admin-dashboard.prd.md` (638 lines, §3–§8 cover v1-AD-d scope) |
| Prerequisite sub-phases merged | ✅ v1-AD-a (`e61f78edf` / PR #72), v1-AD-b (`f03ed1cba` / PR #76), v1-AD-c (`cf89890f3` / PR #81) |
| Substrate in place | ✅ `get_*_opt` accessor family (v1-AD-b), `CONFIG_KEY_METADATA` 61+ entries (v1-AD-a/b), `governance_log_notify` trigger (migration `2026-04-20-000000-0000_add_governance_log_notify`) |
| Decision queue | ✅ zero pending entries (last-resolved DQ #41 from v1-AD-b) |
| Upstream divergence | ✅ fetched — governance-v0 ahead of upstream; no rebase pending before plan |
| Working tree | ⚠ 1 item: untracked runlog + report files from v1-AD-c phase close (not blocking; can be committed as `docs(v1-AD-c): archive runlog + completion report` or left for the plan session to stage) |
| OQs resolved | ✅ OQ-V1-AD-01 (defer HTML), OQ-V1-AD-02 (hand-roll SSE), OQ-V1-AD-03 (dry-run semantics) — all resolved at v1-AD-b time |

## Dependencies already satisfied

- **SSE notification channel** — `pg_notify('governance_events', …)` trigger shipped in `2026-04-20-000000-0000_add_governance_log_notify/up.sql`. v1-AD-d's `/admin/audit/stream` consumes it.
- **Audit list reader** — v1-AD-b shipped `list_admin_config_audit` at task 4. v1-AD-d's dashboard aggregate reuses the read pattern.
- **Capability gates** — `instance_admin` + `community_admin(id)` checks established in v1-AD-b. v1-AD-d reuses.
- **Rule-set version pointer** — `rule_set.active_version_id` wired by v1-AD-c; dashboard aggregate reads it per-community.

## Branch topology at plan-time

```
governance-v0 @ cf89890f3          ← PR #81 merge commit, v1-AD-c landed
      │
      │  (plan PR will branch from here)
      ▼
plan/v1-AD-d                       ← NEW — advisor or planning session cuts this
      │
      │  (plan PR merges back to governance-v0)
      ▼
governance-v0 @ <plan-merge-sha>
      │
      │  (impl session cuts phase-v1-AD-d from here)
      ▼
phase-v1-AD-d                      ← NEW — impl branch for ralph
```

This mirrors the v1-AD-a/b/c precedent exactly (see DQ #40 resolution 2026-04-20 and `project_v1_impl_branches_ready.md`).

## Open issues relevant to v1-AD-d planning

None blocking. Carry-forward issues #82–#85 are v1-AD-c chores and do not gate v1-AD-d.

## Expected plan shape (for planning session)

Looking at v1-AD-b and v1-AD-c as templates, v1-AD-d is **smaller** than v1-AD-b (B had 8 tasks, 3 handlers + accessor family + audit list; D has 2 handlers) and **closer in size to v1-AD-c** (C had 8 tasks mostly wire-up + tests). Expected range: **5–7 tasks**.

Likely task breakdown (planning session will refine):
- Task 0 — plan commit + pre-phase audit
- Task 1 — `/admin/dashboard` aggregate handler + DTO
- Task 2 — `/admin/audit/stream` SSE handler with hand-rolled async-stream + LISTEN/NOTIFY loop
- Task 3 — route wiring for both
- Task 4 — e2e tests (3–4: dashboard happy path, SSE happy path + disconnection, capability gates)
- Task 5 (maybe) — capability snapshot freshness (dashboard must reflect post-write state; may overlap with v1-AD-b's dry-run infrastructure)

## Memory to check at plan-session start

- `project_v1_AD_c_closed.md` — prior-phase-close state (landed commit, carry-forward issues)
- `project_v1_impl_branches_ready.md` — branch topology pattern (note: 3 days old, verify before asserting)
- `feedback_branch_manager_pm_split.md` — PM/impl split protocol continues for v1-AD-d
- `feedback_pr_review_triage_pattern.md` + `feedback_coderabbit_block_merge_critical.md` — CR triage protocol unchanged
- `.claude/rules/governance-log-entry-kind-registry.md` — NO new entry kinds expected in v1-AD-d (dashboard is read-only; SSE is read-only)

## Planning session first command

```bash
# From primary worktree on any branch (advisor suggests governance-v0 @ cf89890f3)
git checkout governance-v0
git pull origin governance-v0
git checkout -b plan/v1-AD-d
# Then in Claude Code:
/prp-plan "v1-admin-dashboard-d — dashboard aggregate + SSE audit stream"
```

The `/prp-plan` skill reads the Brehon design-doc context block automatically (CLAUDE.md Tier-1 command). Point it at the v1-admin-dashboard PRD §4 (HTTP API) + §6 (pages/dashboard) + §8.6 rollout step 2.

## Notes for planning session

- **No new migrations expected.** Dashboard + SSE are both read-only; LISTEN/NOTIFY channel already exists.
- **No new entry_kinds expected.** Per registry rule, dashboard + SSE are consumers of existing `admin_config_changed` / `admin_config_change_denied` kinds, not emitters.
- **Hand-rolled SSE pattern** — new territory for the Brehon fork. Advisor recommends the plan include a §Patterns-to-mirror reference to the simplest upstream SSE example available (axum `Sse<Stream>` + `async_stream::stream!`). If no good Lemmy-local mirror exists, note it as a "novel pattern" risk in §18.
- **Capability test ordering** — v1-AD-b established "check capability first, then read state." v1-AD-d dashboard aggregate should follow the same order (check `instance_admin` or `community_admin(id)` BEFORE any config read).
- **SSE disconnect/reconnect** — clients can drop and reconnect; plan should cover what happens to in-flight events (likely: server drops, client re-fetches via audit-list endpoint for catch-up, then re-subscribes to SSE from "now"). Document in §GOTCHA.

## Advisor confidence

**High.** v1-AD-d is the simplest sub-phase of the v1-AD wave — pure additions, no new substrate, two read-only endpoints, all dependencies shipped.
