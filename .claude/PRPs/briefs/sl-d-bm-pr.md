---
phase: v1-SL-d
role: bm-task
task: bm-pr
brief_n: 1
authored: 2026-05-10
---

# [role:bm-task] v1-SL-d bm-pr — open PR phase-v1-SL-d → governance-v0

## §1 Role + dispatch

`[role:bm-task] bm-pr v1-SL-d`

## §2 Scope

Run `/bm-pr` for phase `v1-SL-d`. Open a PR from `phase-v1-SL-d` into `governance-v0` on `barrie-cork/lemmy`.

**PR title:** `feat(v1-SL-d): sponsor-liability compute/fire split + submit_jury_vote pending transition + e2e tests`

**PR body must include:**
- Summary: SL-d ships the compute/fire split of `apply_sponsor_liability`, the `submit_jury_vote` mutation to transition liability-bearing sponsored cases to `SponsorLiabilityPending`, and 4 e2e tests + 2 unit tests covering all paths.
- Stories: 1 (compute/fire split + wrapper preserves v0), 2 (pending transition + no-sponsor + NoAction paths), 3 (grace_window_for_severity config mapping) — all `[done]`
- Phase 1 workspace checks: DQ #195-#199 all pass
- Phase 2 e2e: 85 passed, 0 failed, 3 ignored (log: e2e-v1-SL-d-78771349e-run2.log)
- Plan reference: `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md`

**Mandatory flags:**
- `--repo barrie-cork/lemmy` (per gh-pr-fork-target.md)
- `--base governance-v0`
- `--head phase-v1-SL-d`
- NOT draft (CodeRabbit skips drafts)

## §3 Required reading

- `.claude/rules/branch-manager.md` — autonomy bounds, file ownership, PR discipline
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/rules/phase-branch.md` — base must be governance-v0, not main
- `.claude/commands/bm/bm-pr.md` — full bm-pr procedure

## §4 Constraints

- Base: `governance-v0`. Never `main`.
- `--repo barrie-cork/lemmy` on every `gh pr` command.
- Not draft.
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`.
- After PR opened: write runlog entry to `.claude/runlog/bm-runlog.md`.
- Push runlog commit to `governance-v0`.
