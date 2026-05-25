# Brief: v1-deps-r1 BM-PR

## 1. Role + dispatch

`[role:bm-task] v1-deps-r1-bm-pr — see .claude/PRPs/briefs/v1-deps-r1-bm-pr-1.md`

## 2. Scope

Open a PR from `phase-v1-deps-r1` → `governance-v0` on `barrie-cork/lemmy`.

**Produce:**
- PR opened (not draft) with title and body per §3 below
- PR number recorded in runlog

**Do NOT:**
- Merge the PR
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `docs/`
- Push any commits to the phase branch

## 3. PR title + body

**Title:** `feat(deps): v1-deps-r1 — diesel-async 0.9 + sha2 0.11 + 8 SemVer-compat bumps`

**Body:**
```
## Summary

- **Task 1** (`7bd047f2d`): diesel-async 0.8 → 0.9 migration — wrapper trait bound rewrite (`AsyncFnOnce + AsyncFunc`) + 42-callsite closure-shape sweep + 30 `scoped_futures` import collapses
- **Task 2** (`16d722c8e`): sha2 0.10 → 0.11 — Cargo pin bump; zero consuming-code changes (API preserved; ADR-012 hash chain byte-identical)
- **Task 3** (`939868483`): 8 SemVer-compat bumps — serde_with 3.20, bcrypt 0.19.1, tokio 1.52.3, rustls 0.23.39, html2text 0.17.1, jsonwebtoken 10.4.0, lettre 0.11.22, rss 2.0.13 (diesel already bumped in T1; dashmap 6.2.x not yet published)

## Validation

- `cargo check --workspace --features full`: ✓ exit 0 (all 3 tasks)
- `cargo clippy --workspace --features full --no-deps -- -D warnings`: ✓ exit 0 (all 3 tasks)
- Phase-tip e2e: ✓ 109 passed; 0 failed (36 min, 2026-05-25)

## Verify report

`.claude/PRPs/reports/v1-deps-r1-verify.md` — all 4 stories ✓

## Plan reference

`.claude/PRPs/plans/v1-deps-r1.plan.md`
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
- Record PR number in `.claude/runlog/` or task output
