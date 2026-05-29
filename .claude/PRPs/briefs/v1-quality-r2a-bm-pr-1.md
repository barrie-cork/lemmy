# Brief: v1-quality-r2a BM-PR

## 1. Role + dispatch

`[role:bm-task] v1-quality-r2a-bm-pr — see .claude/PRPs/briefs/v1-quality-r2a-bm-pr-1.md`

## 2. Scope

Open a PR from `phase-v1-quality-r2` → `governance-v0` on `barrie-cork/lemmy`.

**Produce:**
- PR opened (not draft) with title and body per §3 below
- PR number recorded in runlog or task output

**Do NOT:**
- Merge the PR
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `docs/`, `tests/`, `migrations/`
- Push any commits to the phase branch
- Comment on or review the PR (separate confirm-gated verb)
- Send Telegram pings (separate confirm-gated verb)

## 3. PR title + body

**Title:** `feat(quality): v1-quality-r2a — DQ duration lint + C3 deferral DQ (#157, #158)`

**Body:**
```
## Summary

- **Task 1** (`18bb926ed`): add `scripts/brehon/dq-lint-durations.sh` (29 LOC composite-id-aware DQ negative-duration linter) + `scripts/brehon/precheck.sh` (8 LOC wrapper that sources the lint as first gate) + sweep 3 back-dated DQ entries to Option A floor — closes #157
  - Floored: id=315 (`08:53Z` → `10:45:00Z`), id=1b8527b076d4-001 (`19:11:20.705989Z` → `19:30:00Z`), id=81719cf8ca8d-001 (`07:08:50.884428Z` → `19:10:00Z`)
- **Task 2** (`0bbc3502d`): C3 deferral DQ — single new `kind: log + from: planner + answered_by: planner` entry `dd6012873857-001` in `resolved[]` for Issue #158 (#159's `emit_reputation_event` helper extraction)
  - Trigger condition for v1-quality-r3 re-entry: any sub-phase introduces a 3rd reputation-event emit path. Two existing emitters byte-identical at `admin_emergency_remove.rs:448` + `submit_jury_vote.rs:1096`.

## Validation

- `bash scripts/brehon/dq-lint-durations.sh`: ✓ exit 0 on post-sweep `.claude/decision-queue.json`
- `bash scripts/brehon/precheck.sh`: ✓ exit 0
- Worker self-tests on both tasks: exit 0
- **No e2e gate** — r2a touches zero Rust per Plan §15.4

## Verify report

Plan §15.3 cross-cutting verification — all 9 checks passed:
- R5: Task 0 enumerated all 9 probes
- R8: `--workspace --features full` uniformly (no `-p <crate>` mixing)
- R9: every cargo gate uses `bash scripts/brehon/cargo-*.sh`
- R10: every cargo invocation redirects to a file
- `dq-lint-durations.sh` exits 0 post-sweep; exits non-zero on synthetic back-dated fixture
- `precheck.sh` sources `dq-lint-durations.sh` via `SCRIPT_DIR`
- Zero edits to files outside §11 list

## Retro

`.claude/PRPs/reports/v1-quality-r2a-retro.md` — gate-6 signed off 2026-05-29 by user.

## Plan reference

`.claude/PRPs/plans/v1-quality-r2a.plan.md`

## Issues addressed

- **Closes #157** at merge time (PR description includes `Closes #157`).
- **Issue #158 — defer-comment to be filed at merge time; issue stays OPEN** as v1-quality-r3 re-entry point per DQ `dd6012873857-001` trigger condition.
```

## 4. Required reading

- `.claude/rules/branch-manager.md` — file ownership boundaries, autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/rules/phase-branch.md` — base MUST be `governance-v0`, not draft

## 5. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- Base branch: `governance-v0` (NOT `main`)
- Not draft (CodeRabbit skips drafts)
- No force-push
- Record PR number in `.claude/runlog/v1-quality-r2a-runlog.md` or task output
- Do NOT close Issue #158 in the PR description (only #157)
