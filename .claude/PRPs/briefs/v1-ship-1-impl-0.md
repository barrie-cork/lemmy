---
phase: v1-ship-1
role: impl-task
task: 0
brief_n: 0
plan: .claude/PRPs/plans/v1-ship-1-r1.plan.md
created: 2026-05-16
related_dq: null
---

# [role:impl-task] v1-ship-1 task 0 — pre-flight harness audit — see .claude/PRPs/briefs/v1-ship-1-impl-0.md

## §1 Role + dispatch

`[role:impl-task] v1-ship-1 task 0 pre-flight harness audit`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 0
from `.claude/PRPs/plans/v1-ship-1-r1.plan.md` §13 (Task 0, lines
~648-716). Verification only — environment + topology sanity for the
AGPL §13 source-disclosure surface delivery.

## §2 Scope

Run Probes 0–11 from plan §13 Task 0 **in order**. Report each probe
result (PASS / FAIL / WARN) with its actual output. **No commit at
Task 0 — verification only. No files changed. No code authored.**

The 12 probes (verbatim from plan §13 Task 0 — run exactly as written
there; the plan's §13 Task 0 "Probes" fenced block is the contract):

- **Probe 0** — Docker daemon running (`docker ps`). Needed only for
  ad-hoc local e2e; Shape G suspended means e2e runs locally later, so
  Docker presence still matters. FAIL exit 1 → blocker.
- **Probe 1** — branch is `phase-v1-ship-1`. FAIL exit 1 → **blocker
  (STOP)**.
- **Probe 2** — working tree clean (`git status --porcelain` empty).
  FAIL → WARN (a lane-advisor commit may be in flight; report the
  dirty paths, do not stop).
- **Probe 3** — `git merge-base --is-ancestor governance-v0 HEAD`
  (phase branch descends from trunk). FAIL exit 1 → blocker.
- **Probe 4** — both workflow YAMLs present
  (`cargo-validate-workspace.yml` + `cargo-test-e2e.yml`). FAIL → WARN
  (Shape G is suspended repo-wide until 2026-06-01 — the YAMLs may be
  disabled/renamed; report presence/absence, do not stop on absence).
- **Probe 5** — `AGPL-NOTICE.md` at repo root, > 1000 bytes. FAIL exit
  1 → **blocker (STOP)** — this file is the load-bearing `include_str!`
  target for Task 3; absence breaks the whole deliverable.
- **Probe 6** — `pub struct GetSiteResponse {` present in
  `crates/db_views/site/src/api.rs` (drift ±20 lines OK; symbol
  presence is the contract). Absent → blocker.
- **Probe 7** — `captcha_enabled: is_captcha_plugin_loaded(),` present
  in `crates/api/api_crud/src/site/read.rs` (drift ±10 lines OK).
  Absent → blocker.
- **Probe 8** — exactly **2** `pub async fn bootstrap() -> LemmyResult<(`
  in `crates/server/tests/e2e.rs` (DQ #226 sanity). Count ≠ 2 →
  **blocker (STOP)** — a 3rd sibling would invalidate the Task 4
  bootstrap choice.
- **Probe 9** — first `pub async fn bootstrap()` line number near 801
  (drift ±50 OK; symbol presence is the contract). Report the actual
  line number.
- **Probe 10** — `federated_instances` sibling route present in
  `crates/api/routes/src/lib.rs` (drift OK). Absent → WARN.
- **Probe 11** — concurrent-PR check (the `gh pr list` jq from plan
  §13 Task 0). Non-empty output → **blocker (STOP)** — a file-ownership
  conflict with another open PR; surface the colliding PR number.

**Blocker probes (STOP + file `kind: "blocker"` DQ, `from: "impl"`):**
Probe 1, Probe 3, Probe 5, Probe 6, Probe 7, Probe 8, Probe 11.
**All other probe failures: report as WARN, continue.** (Rationale:
Probe 2 dirty-tree and Probe 4 workflow-absence are expected
side-effects of the active lane session + the repo-wide Shape G
suspension; they must not halt the pre-flight.)

## §3 Required reading

In this order:

1. **`.claude/PRPs/plans/v1-ship-1-r1.plan.md` §13 Task 0** (lines
   ~648-716) — the verbatim 12-probe fenced block. Run those exact
   commands; this brief's bullet list is orientation, the plan block
   is the contract.
2. **`.claude/rules/pre-phase-harness-audit.md`** — R5 audit shape
   (enumerate ALL probes explicitly; no silent skips).
3. **`.claude/rules/decision-queue.md`** — Recipe 1 (blocker DQ) +
   "Mid-task visibility" (commit+push the DQ immediately if filed).

## §3a Handover from prior cohort

(none — Task 0 is the first task; no prior cohort)

## §4 Constraints

- **No commit.** Task 0 is verification only — no files changed, no
  git commit, no worktree branch tip movement beyond what the daemon
  cuts.
- **No code authoring.** Do not edit `crates/**`, `migrations/**`,
  `tests/**`, `docs/**`, `.claude/**`.
- **DQ mid-task push:** if a blocker DQ is filed, `git add
  .claude/decision-queue.json && git commit -m "chore(decision-queue):
  impl raised DQ #<id> — <slug>" && git push origin <worktree-branch>`
  immediately per `decision-queue.md` "Mid-task visibility". Compute
  `next_id` across `.claude/decision-queue.json` +
  `.claude/decision-queue-archive-*.json` (DQ #50 collision lesson).
- **Attribution:** DQ entries use `from: "impl"`, `answered_by: null`.
  NEVER `from: "advisor"` / `answered_by: "advisor"` / `answered_by:
  "user"`. NEVER `kind: "clarify"` (advisor-only).
- **Branch:** must be on `phase-v1-ship-1` (Probe 1 verifies; if not,
  that IS the blocker).
- **Shape G note:** Task 0 has NO cargo invocation — pure
  topology/file-presence probes. The repo-wide Shape-G suspension
  (DQ #228/#229, until 2026-06-01) does not affect Task 0; it only
  affects Tasks 1-4's validation handoff (those write
  `kind: "validate-pending-laptop"`, not this task).

## §4.1 CANONICAL CASE OVERRIDE

Not applicable — Task 0 authors no e2e test. No `LemmyResult` /
`Box<dyn Error>` Case A/B/C decision required (that is Task 4's
concern, pre-resolved as Case A in plan §10.6 GOTCHA 1).

## §6 Expected output (return to advisor)

```
## Task 0 complete — v1-ship-1 pre-flight harness audit

**Probes 0-11:**
  - Probe 0 (Docker): PASS/FAIL
  - Probe 1 (branch=phase-v1-ship-1): PASS/FAIL
  - Probe 2 (tree clean): PASS/WARN — <dirty paths if WARN>
  - Probe 3 (descends governance-v0): PASS/FAIL
  - Probe 4 (workflow YAMLs): PASS/WARN — <present? Shape-G-suspended note>
  - Probe 5 (AGPL-NOTICE.md >1000B): PASS/FAIL — <byte count>
  - Probe 6 (GetSiteResponse struct): PASS/FAIL — <line>
  - Probe 7 (read_site captcha_enabled): PASS/FAIL — <line>
  - Probe 8 (exactly 2 bootstraps): PASS/FAIL — <count>
  - Probe 9 (governance bootstrap line): PASS — <actual line ~801>
  - Probe 10 (federated_instances route): PASS/WARN — <line>
  - Probe 11 (concurrent-PR check): PASS/FAIL — <colliding PRs or empty>
**Verdict:** ALL PASS (proceed to Task 1) | BLOCKER on Probe <N> (DQ #<id>)
**No commit** (Task 0 verification only).
**Next:** advisor queues Task 1 (DTOs — SourceDisclosure/GetSource/GetSourceResponse).
```

Plus the DQ #N reference if a blocker was filed.

## §7 Why this brief differs from the plan

No overrides. Clean execution of plan §13 Task 0. The WARN-vs-blocker
split in §2/§4 is an explicit codification of the plan's intent (the
plan marks Probe 2/4/10 as soft and Probe 1/5/8/11 as hard via their
`exit 1` vs comment-only handling); this brief makes the split
unambiguous for the subagent. The Shape-G-suspension context is
session state from the bootstrap, not a plan deviation — Task 0 runs
no cargo so the suspension is immaterial here.
