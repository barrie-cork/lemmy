---
phase: v1-AD-e
role: bm-task
task: bm-pr
brief_n: 1
authored: 2026-05-17
---

# [role:bm-task] AD-e bm-pr — open PR for phase-v1-AD-e — see .claude/PRPs/briefs/v1-AD-e-bm-pr-1.md

## §1 Role + dispatch

`[role:bm-task] AD-e bm-pr — open PR for phase-v1-AD-e into governance-v0`

## §2 Scope

Run `bm-pr` for phase-v1-AD-e. AD-e is **server-rendered admin dashboard
HTML pages** (Dashboard + Audit) — pure-templating over the already-shipped
v1-AD-d JSON endpoints. All 6 plan tasks delivered + validated (Task 4
pre-satisfied via Task-2 scope-bleed; Task 0 read-only no-commit by design).

**Phase branch:** `phase-v1-AD-e`
**Tip:** `efc0d72e1` (`/brehon-verify` report finalize)
**Base:** `governance-v0`
**Repo:** `barrie-cork/lemmy`

**No completion report, no retro yet** — the v1-AD-e plan defers retro to
**post-merge (user gate 6)**. The PR body assembles from:
- the plan: `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md`
- the verify report: `.claude/PRPs/reports/v1-AD-e-verify.md` (already on
  phase branch, committed `efc0d72e1`) — use its story table + outcome
  summary as the `## Summary` content
- the commit log: `git log governance-v0..HEAD --oneline`

**Open the PR only. Do NOT merge** (merge is a separate verb behind user
gate 5).

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` — bm-pr operational script (follow phases
  1→7 verbatim)
- `.claude/rules/branch-manager.md` — BM autonomy bounds + file-ownership
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
- `.claude/rules/phase-branch.md` — phase-branch + PR flow (base
  `governance-v0`, never `main`; not draft — CR skips drafts)
- `.claude/PRPs/reports/v1-AD-e-verify.md` — verify report → PR body summary
- `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` — plan → title + plan
  reference

## §4 Constraints

- **`--repo barrie-cork/lemmy`** on every `gh` command (forks default to
  upstream `LemmyNet/lemmy` without it).
- Base: `governance-v0` (NOT `main`). Head: `phase-v1-AD-e`.
- **Not draft** (CodeRabbit skips drafts per `phase-branch.md`).
- **PR title:** `Phase v1-AD-e — server-rendered admin dashboard HTML pages (Dashboard + Audit)`
- **PR body** assembles from (in this order):
  - `## Summary` — one paragraph derived from the verify report outcome
    summary + the 3 §16a stories (all ✓)
  - `## Plan reference` — `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md`
  - `## Completion report` — `Pending — retro deferred to post-merge (user gate 6)`
  - `## Task table` — Tasks 0–5 with their commits + DQ entries:
    - Task 0 — pre-flight harness audit — (no commit; read-only by plan design) — PASS
    - Task 1 — maud 0.27.0 engine dep — `55c373016` — DQ #241 pass
    - Task 2 — gather_dashboard extract + dashboard HTML + `/dashboard/view` route — `ade904851` — DQ #242 pass
    - Task 3 — `/audit/view` route + admin_audit_html import — `5afe5b378` — DQ #243 pass
    - Task 4 — audit live-tail `<script>` — (no commit; **pre-satisfied** in Task 2 `ade904851` via scope-bleed; AUDIT_SCRIPT verified 8/8 spec-conformant) — PASS
    - Task 5 — e2e test (admin 200 / non-admin 403 / flag-off 404) — `460214d5e` — DQ #244 pass
  - `## Commits` — `git log governance-v0..HEAD --oneline` output, one
    bullet per line
  - `## Validation` — note: all DoD gates exit 0 across every task; full
    e2e run on advisor-laptop (Docker, 1906.55s): **93 passed / 0 failed /
    5 ignored**, all 4 new v1-AD-e tests `ok`; `/brehon-verify` 3/3
    stories ✓
  - `## Closes` — extract any `closes #N` / `fixes #N` from commit
    messages (likely none — v1-AD-e is plan-driven, not issue-driven; if
    none, write `(none — plan-driven sub-phase)`)
- **Phase 1b (DQ historical-fail sweep):** run it; it is a **no-op**
  (DQ pending = 0, zero failure-enum entries). Do not skip the step —
  just expect "swept 0 entries".
- **Phase 1c (Phase 2 e2e gate):** the plan touches `crates/server/tests/e2e.rs`
  so the gate fires. It is **satisfied** — DQ #244
  (`kind: validate-pending-laptop-e2e`, `branch: phase-v1-AD-e`,
  `result: pass`, 93/0/5) is in `resolved[]`. The gate's Python will
  pass. Do NOT surface user gate 4 — it was already cleared (advisor ran
  e2e locally as the Task 5 DoD; user-gate-4 local-vs-dispatch choice was
  made at Task 5 dispatch). If the gate Python somehow STOPs, file a
  `kind: "blocker"` DQ (do NOT improvise) — but it should pass.
- Write `.claude/PRPs/reviews/pr-<N>-findings.yaml` shell per
  `.claude/PRPs/reviews/SCHEMA.md` for the later CR-triage cycle.
- Append the Phase 6 runlog entry to `.claude/runlog/bm-runlog.md`
  (`## bm: PR opened — <ISO>`).
- Do **NOT** post a Telegram ping (advisor handles outbound; no ping
  this phase unless the user asks).
- Do **NOT** touch `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**` (BM file-ownership HARD boundary).
