# GitHub Actions Minutes Audit — 2026-05-15

**Trigger:** GitHub notification — 2800 of 3000 free minutes consumed (~200 remaining).  
**Authored:** 2026-05-15 by advisor session (investigation only; no workflow changes landed).

---

## 1. Context

The repo `barrie-cork/lemmy` is **private**. GitHub bills private-repo Actions at **2× multiplier** for Linux runners — every 1 minute of wall-clock = 2 billable minutes. The free tier for personal accounts is 3000 min/month.

We have ~200 billable minutes remaining and 17 days until June 1 (≈12 min/day headroom).

---

## 2. Active Workflows

| Workflow | ID | State | Trigger |
|---|---|---|---|
| `adr-compliance` | 262402017 | active | PR open/sync/reopen + push to `phase-v1-SL-b`, `phase-v1-SL-a` |
| `cargo-validate-workspace` | 267337786 | active | Push to `junior/*` (paths filter on crates/migrations/Cargo) |
| `cargo-validate-migration` | 267337787 | active | Push to `junior/*` (migration paths) |
| `cargo-test-e2e` | 262948095 | active | workflow_dispatch only (auto-trigger removed 2026-04-28) |
| `Copilot code review` | 265500620 | active | Platform-managed; fires on every PR open/sync |
| `Dependabot Updates` | 266438818 | active | Scheduled by GitHub; not controllable |
| `adr-drift` | 262555944 | active | workflow_dispatch only (schedule disabled 2026-04-25) |
| `oq-sweep` | 262555945 | active | workflow_dispatch only (schedule disabled 2026-04-25) |
| `plan-drift` | 262555946 | active | workflow_dispatch only (schedule disabled 2026-04-25) |

`claude.yml`, `claude-code-action.yml`, `claude-code-review.yml`, `governance-ai-review.yml` — all `.disabled` on disk (not registered).

---

## 3. Run Volume (last 200 runs, approx. last 7 days)

| Workflow | Run count | Est. wall-clock | Est. billable min |
|---|---|---|---|
| `cargo-validate-workspace` | 52 | ~20 min each (warm cache) | ~2080 (if all completed; many cancelled) |
| `adr-compliance` | 27 | ~3 min each | ~162 |
| `Copilot code review` | 9 | ~2–5 min each | ~54–90 |
| `cargo-validate-migration` | 8 | ~5 min each | ~80 |
| `Dependabot Updates` | 4 | ~2 min each | ~16 |

**`cargo-validate-workspace` is the dominant consumer.** The `cancel-in-progress: true` concurrency group means many runs are cancelled early (3 cancelled today alone), but each cancellation still bills for time elapsed before cancellation.

---

## 4. Day-by-Day Breakdown (recent spike)

| Date | Total runs | Breakdown |
|---|---|---|
| 2026-05-15 | 23 | 12× adr-compliance, 5× Copilot, 5× cargo-validate-workspace, 1× cargo-validate-migration |
| 2026-05-14 | 2 | 1× cargo-validate-workspace, 1× cargo-validate-migration |
| 2026-05-13 | 10 | 8× adr-compliance, 1× cargo-validate-workspace, 1× Copilot |
| 2026-05-12 | 16 | 13× cargo-validate-workspace, 1× adr-compliance, 1× cargo-validate-migration, 1× Copilot |
| 2026-05-11 | 11 | 4× adr-compliance, 4× Dependabot, 3× cargo-validate-workspace |
| 2026-05-10 | 37 | 28× cargo-validate-workspace, 5× cargo-validate-migration, 2× adr-compliance, 2× Copilot |
| 2026-05-09 | 9 | 8× cargo-validate-workspace, 1× cargo-test-e2e |

**2026-05-10 was the largest single day (37 runs)** — corresponds to a high-volume Junior task dispatch day (multiple cohort members pushing `junior/*` branches in sequence or parallel).

---

## 5. Root Causes

### 5.1 Today's spike (2026-05-15, 23 runs)

The 6-PR refactor blitz (PRs #128–#132, the audit fix-before-next-phase tier) generated:
- **adr-compliance:** 2–3 runs per PR (opened + synchronize events) × 6 PRs = ~12–18 runs
- **Copilot code review:** 1 run per PR × 5 PRs = 5 runs
- **cargo-validate-workspace:** Junior worker for PR-1 (`chore/refactor-e2e-error-types`) pushed multiple times (impl + continuation pass) = 3–5 runs (3 cancelled by concurrency group)

### 5.2 Structural driver: `cargo-validate-workspace` on `junior/*`

This is Shape G (shipped PR #104, 2026-04-27). Every impl-task Junior worker that pushes to a `junior/*` branch triggers a workspace check (~20 min cold, ~8 min warm cache). A single sub-phase with 10 impl tasks = 10+ workspace runs = 200+ billable minutes alone. The `cancel-in-progress` group mitigates re-pushes on the same branch but does not reduce the per-task cost.

### 5.3 `adr-compliance` trigger is stale

The `push.branches` list in `adr-compliance.yml` still contains `phase-v1-SL-b` and `phase-v1-SL-a` — both dead branches. These entries are harmless now (no pushes to those branches) but represent config drift. The PR trigger is the load-bearing path.

### 5.4 Copilot code review — platform-managed, not disableable via API

`gh api --method PUT .../workflows/265500620/disable` returns HTTP 422. GitHub does not expose a disable endpoint for platform-managed workflows. The only control is preventing PR open/sync events.

---

## 6. Options to Investigate

These are options only — not decisions. Each has tradeoffs.

### Option A — Pause impl work until June 1
No new Junior task dispatch = no new `cargo-validate-workspace` runs. No new PRs = fewer adr-compliance + Copilot runs. Cost: 17 days of impl freeze.

### Option B — Disable `adr-compliance` temporarily
`gh api --method PUT .../workflows/262402017/disable` (confirmed working). Saves ~3 min per PR open/sync. Downside: lose ADR red-flag coverage during any impl work done before June 1. Re-enable on June 1: `...enable`.

### Option C — Remove stale push triggers from `adr-compliance.yml`
Edit `push.branches` to remove `phase-v1-SL-b` and `phase-v1-SL-a`. Small cleanup, no functional impact today (those branches are dead), but good hygiene. Does NOT reduce PR-triggered runs.

### Option D — Upgrade GitHub plan before June 1
- GitHub Pro ($4/month): 2000 extra min → 5000 total/month
- Extra minutes purchase: $0.008/min (Linux) — 200 min ≈ $1.60 one-off
- This is the zero-disruption option.

### Option E — Make `cargo-validate-workspace` conditional
Add a `workflow_dispatch` path or restrict to branches matching a tighter pattern. Currently fires on all `junior/*` pushes. Could restrict to e.g. only the last push per Junior task (harder to implement; Junior pushes mid-task for DQ visibility).

### Option F — Move repo back to public temporarily
Public repos get 2000 free min/month on the Free plan but NO multiplier (1 min wall-clock = 1 billable min). At public + no multiplier, remaining ~200 min = ~200 wall-clock min. Not recommended (AGPL §13 + privacy implications).

---

## 7. Observations for Investigation

1. **The multiplier is the key lever.** Private Linux = 2×. Making the repo public removes the multiplier but has AGPL implications (per `project_brehon_agpl_repo_privacy.md` — §13 triggers at first external user, not at public visibility alone).
2. **`cargo-validate-workspace` dominates.** Shape G was chosen to protect the EliteDesk; the tradeoff is Actions minutes. At active development pace (~10 Junior tasks/week), Shape G consumes ~200–400 billable minutes/week on its own.
3. **The free tier (3000 min) is undersized for the current development pace.** Even with no refactor blitz, a normal impl sub-phase with 10 tasks × 20 min each = 200 wall-clock min × 2 = 400 billable min. Three sub-phases/month = 1200 min from impl alone, before adr-compliance and Copilot.
4. **June cadence question:** if v1-ship-1 (the next planned sub-phase per the parked PRD) proceeds in June, the same pattern will recur immediately. The structural fix is either plan upgrade or per-task run optimisation.
