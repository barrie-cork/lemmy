> **[CLOSED — shipped 2026-05-24, advisory-lock phase]** This brief is from the
> completed `v1-federation-inbound-e` sub-phase (pg_advisory_xact_lock eviction fix).

# BM-cut brief — v1-federation-inbound-e

**Role:** `[role:bm-task]`
**Phase:** `v1-federation-inbound-e`
**Authored:** 2026-05-22
**Base:** `governance-v0` @ `f4d584345`

---

## 1. Role + dispatch line

```
[role:bm-task] v1-federation-inbound-e bm-cut — see .claude/PRPs/briefs/v1-federation-inbound-e-bm-cut-1.md
```

---

## 2. Scope

### 2.1 What to produce

1. Create branch `phase-v1-federation-inbound-e` off `governance-v0` HEAD (`f4d584345`).
2. Push `phase-v1-federation-inbound-e` to `origin`.
3. Append one line to `.claude/runlog/bm-runlog.md`:
   ```
   ## bm-cut: phase-v1-federation-inbound-e off f4d584345 — 2026-05-22
   ```
4. Commit + push the runlog append to `governance-v0`.

### 2.2 Scope boundary

**IN scope:**
- `git checkout -b phase-v1-federation-inbound-e` from `governance-v0`
- `git push -u origin phase-v1-federation-inbound-e`
- `.claude/runlog/bm-runlog.md` one-line append on `governance-v0`

**OUT of scope:**
- Any edit to `crates/**`, `migrations/**`, `tests/**`
- Any PR creation (bm-pr is a later verb)
- Any DQ entries
- Any worktree creation (that is the advisor/user's responsibility after bm-cut)

---

## 3. Required reading

- `.claude/commands/bm/bm-cut.md` — canonical bm-cut procedure
- `.claude/rules/branch-manager.md` — file ownership + autonomy bounds
- `.claude/rules/phase-branch.md` — phase branch discipline

---

## 4. Constraints

- **PRE-PUSH MANDATE:** confirm `git status --short` is clean on `governance-v0` before cut.
- **Base SHA verify:** `git rev-parse --short governance-v0` MUST equal `f4d584345` before cut. If drift detected, STOP and file `kind: "blocker"` DQ.
- **Branch name:** exactly `phase-v1-federation-inbound-e`. No deviation.
- **Push target:** `origin` (barrie-cork/lemmy). Use `--repo barrie-cork/lemmy` on any `gh` call.
- **Runlog append:** to `.claude/runlog/bm-runlog.md` on `governance-v0`, one line only. Subject: `chore(bm): cut phase-v1-federation-inbound-e`.
- **Attribution:** `from: "bm"` on any DQ entries. Never `answered_by: "advisor"`.
