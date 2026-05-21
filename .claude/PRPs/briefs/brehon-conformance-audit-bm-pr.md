---
phase: brehon-conformance-audit
role: bm-task
task: bm-pr
brief_n: 1
authored: 2026-05-21
canonical_ref: .claude/PRPs/briefs/federation-inbound-a-bm-pr.md
---

# [role:bm-task] brehon-conformance-audit bm-pr — open PR for phase-brehon-conformance-audit — see .claude/PRPs/briefs/brehon-conformance-audit-bm-pr.md

## §1 Role + dispatch

`[role:bm-task] brehon-conformance-audit bm-pr — open PR for phase-brehon-conformance-audit into governance-v0`

## §2 Scope

Run `bm-pr` for `phase-brehon-conformance-audit`. This sub-phase delivers the **Phase-6 convention-divergence detection skill + Clippy enforcement gate**. The skill detects new federation governance code diverging from canonical-sibling patterns along six axes; the Clippy gate enforces the axis-4 (error-idiom) sub-rule structurally via `clippy.toml` + workspace-allow override + per-module `#![deny(clippy::disallowed_methods)]`.

**Phase branch:** `phase-brehon-conformance-audit`
**Tip:** `425ab13c8` (docs(retro): Task 13 retro authored + lesson promoted; daemon == origin == lane)
**Base:** `governance-v0`
**Repo:** `barrie-cork/lemmy`

**Validation status (for PR body):**
- Cohort 1: Task 1 (SKILL.md skeleton) ✓
- Cohort 2 [P]: Tasks 2 (six axis sub-files), 3 (find-sibling.sh), 4 (audit-metrics.schema.json), 5 (METRICS.md), 10 (rust-analyzer-mcp + .mcp.json.example) — DQ #300/#301/#302/#304/#305 pass
- Cohort 2.5: Task 6 (compute-metrics.sh) — DQ #316 pass
- Cohort 2.6: Task 8a (revert prior broken clippy.toml per DQ #311 mechanism revision) — DQ #312 pass
- Cohort 2.7: Task 8 v2 (corrected mechanism: clippy.toml + Cargo.toml workspace-allow + per-module deny direction per rustc rule 4) — DQ #314 pass (post advisor-direct TOML inline-table fix at `3082c35ff`)
- Cohort 3: Task 7 (dogfood v1-federation-inbound-b two snapshots) — DQ #317 pass. **Calibration metrics: axis-4 precision 1.000, recall 1.000, latent-footgun catch rate 1, lead time 27.5h.**
- Cohort 4: Task 9 (per-module #![deny] on 3 federation mod.rs) — DQ #315 pass
- Cohort 5 [P]: Tasks 11 (advisor-orchestrator.md §3.1.1 + §3.9.1 + §G4 wiring) + 12 (two paired lesson files) — DQ #318 + #319 pass
- Cohort 6: Task 13 retro authored at `425ab13c8` (advisor-side per `feedback_retro_not_report.md`); promoted lesson `feedback_clippy_per_module_deny_requires_workspace_allow.md` (rustc lint-precedence rule 4 codification)

**Phase-2 e2e:** N/A — PRECON-2 OUT (markdown + bash + attribute-only Rust; no test execution required per plan §14).

**Retro is on the phase branch** at `.claude/PRPs/reports/brehon-conformance-audit-retro.md` (authored pre-PR at User Gate 6 sign-off 2026-05-21). The PR body MUST reference it.

**fix-impl history** (DQ #311 mechanism revision):
- fix-impl-1: `b00be611a` (lemmy_utils 6-site unwrap_or_default replacement — net-positive cleanup)
- fix-impl-3: `34f5cc567` (lemmy_diesel_utils 4-site fix — net-positive cleanup)
- Task 8 v1 (`c3aaba47f`) — REVERTED at `bf92be5dc` (Task 8a) per mechanism revision
- Task 8 v2 (`6720dc72a` + TOML fix `3082c35ff`) — corrected mechanism shipped

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` — bm-pr verb
- `.claude/rules/branch-manager.md` — BM autonomy bounds + file-ownership
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
- `.claude/rules/phase-branch.md` — phase-branch + PR flow (base governance-v0, NOT main; NOT draft)
- `.claude/PRPs/plans/brehon-conformance-audit.plan.md` — plan reference for PR body
- `.claude/PRPs/reports/brehon-conformance-audit-retro.md` — retro authored at User Gate 6 (this PR body cites it)
- `.claude/lessons/feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` — runlog timing (write runlog entry POST-merge only; do NOT commit a gov-v0 runlog entry before gh pr merge)

## §4 Constraints

- **--repo barrie-cork/lemmy** on every `gh` command (fork default is upstream LemmyNet/lemmy).
- Base: `governance-v0` (NOT `main` — main is upstream-rebase only).
- Head: `phase-brehon-conformance-audit`.
- **Not draft** (CodeRabbit skips drafts).
- **PR title:** `Phase brehon-conformance-audit — Phase-6 convention-divergence skill + Clippy gate (axis-4 precision/recall 1.000 + 27.5h lead time)` (≤ keep under ~150 chars; trim suffix if gh rejects).
- **PR body** assembles from:
  - 1-line summary: "Brehon skill detecting Phase-6 federation convention-divergence (six axes) + Clippy structural enforcement of axis-4 (error-idiom) via workspace-allow + per-module deny direction. Dogfood proved axis-4 precision/recall 1.000 on v1-federation-inbound-b two snapshots."
  - Task summary table: copy the cohort × DQ table from §2 above (or summarise tasks 1-13 with DQs and final commit SHAs).
  - Plan reference: `.claude/PRPs/plans/brehon-conformance-audit.plan.md`.
  - **Retro reference:** `.claude/PRPs/reports/brehon-conformance-audit-retro.md` — link prominently; the retro covers the cycle-3 catch-fire recovery + 5 forward watch-items.
  - Dogfood metrics block (verbatim from §2 + retro §1):
    ```
    axis-4 precision: 1.000
    axis-4 recall: 1.000
    Lead time (median): 27.5h after ground-truth event (fix-impl-3 at 8b04e69a6)
    Latent-footgun catch rate (axis-4): 1
    ```
  - DQ #311 mechanism revision note — the Task 8 v1 cycle-3 catch-fire + mechanism inversion (workspace-allow + per-module deny via rustc rule 4) is the load-bearing technical decision; cite the retro §2 for detail.
  - `git log governance-v0..phase-brehon-conformance-audit --oneline` shows 89 commits — include as a collapsed `<details>` block, NOT raw paste. Per-task summary table is preferable.
  - Test plan checklist (markdown checkboxes):
    - [x] All §13 task validate-pending-laptop DQs resolved pass (10 DQs: #300, #301, #302, #304, #305, #306, #312, #314, #315, #316, #317, #318, #319)
    - [x] Dogfood calibration test passed (axis-4 precision/recall 1.000, lead time 27.5h)
    - [x] Three federation mod.rs `#![deny(clippy::disallowed_methods)]` enforced + workspace-wide `cargo clippy --workspace --features full` clean at phase tip
    - [x] Retro authored + lesson promoted
    - [ ] CR auto-review triage (next: `bm-poll-cr` + `bm-triage`)
- **L14 runlog discipline:** do NOT commit a `bm-runlog.md` entry to `governance-v0` as part of this `bm-pr` verb in a way that will conflict with the post-merge runlog write. Per `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr`: if bm-pr writes a phase-branch runlog entry, that is fine (it merges with the PR); the trunk runlog entry is written COMPLETE POST-merge only.
- **Do NOT merge the PR** — this verb opens the PR only. Merge is User Gate 5 + a separate `bm-merge` dispatch.
- Write `.claude/PRPs/reviews/pr-<N>-findings.yaml` shell (per SCHEMA.md) so the later `bm-poll-cr` + `bm-triage` have a target.
- Report the opened PR number + URL in the task output.

## §5 Out of scope

- Merging the PR (User Gate 5 / `bm-merge`).
- CodeRabbit triage (separate `bm-poll-cr` + `bm-triage` cycle).
- Any edit to `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**` (BM file-ownership boundary — per `.claude/rules/branch-manager.md`).
- Sync of new lesson files to PMD — that runs post-merge on `governance-v0` per retro §8 recipe; not part of this bm-pr verb.

## §6 Commit subject

The BM Junior's own commit (if any — typically only for the findings YAML shell + phase-branch runlog entry) uses `chore(bm): ` prefix per branch-manager.md autonomy rules.
