# Brief: bm-pr — v1-RT-r2

## 1. Role + dispatch

`[role:bm-task] v1-RT-r2 bm-pr — open PR phase-v1-RT-r2 → governance-v0 — see .claude/PRPs/briefs/v1-RT-r2-bm-pr-1.md`

## 2. Scope

Open a PR from `phase-v1-RT-r2` into `governance-v0` on `barrie-cork/lemmy`.

- Base: `governance-v0`
- Head: `phase-v1-RT-r2`
- Repo flag: `--repo barrie-cork/lemmy` (mandatory per `.claude/rules/gh-pr-fork-target.md`)
- Not draft (CodeRabbit skips drafts)

## 3. PR title and body

**Title:** `v1-RT-r2 — per-dimension chained-halving decay + bounds clamp behind feature flag`

**Body:**

```
## Summary

- `compute_applied_delta` extended with `v1_enabled: bool` parameter; v1 path applies per-half-life chained right-shift (floor(age/hl) halvings, capped at 31); v0 path preserved verbatim
- `chained_halve` private helper added
- `recompute_snapshot` reads `feature.reputation_v1_decay_enabled` flag; when true, resolves per-(dimension, direction) half-life via `resolve_half_life_for_event` and applies per-dimension bounds clamp via `clamp_dimension_i32`
- 11 new synchronous unit tests; all 15 unit tests pass; e2e 108 passed (flag=false default → no behavioural change)

## Validation

- cargo-check --workspace --features full: exit 0
- cargo-clippy --workspace --features full --no-deps -- -D warnings: exit 0
- cargo-test --workspace --features full --lib -- reputation_snapshot::tests: 15 passed
- cargo-test --workspace --test e2e --features full: 108 passed, E2E_EXIT_0

## Plan reference

`.claude/PRPs/plans/v1-RT-r2.plan.md`

## Completion report

`.claude/PRPs/reports/v1-RT-r2-retro.md`
`.claude/PRPs/reports/v1-RT-r2-verify.md` — all 3 stories ✓

## Key commits

- `c963c9066` feat(rep-tuning): replace compute_applied_delta with chained halving (task 1)
- `1bdd51280` feat(rep-tuning): wire v1-RT-r2 flag + resolver + clamp + 11 tests (task 2)
```

## 4. Constraints

- Use `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-RT-r2`
- Do NOT open as draft
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`
- After PR is created: append a `bm:` runlog line to `.claude/runlog/bm-runlog.md` with PR number and timestamp
- Commit the runlog update on `phase-v1-RT-r2` and push
