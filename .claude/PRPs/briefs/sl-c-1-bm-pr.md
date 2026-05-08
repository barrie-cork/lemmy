# bm-pr brief — open PR phase-v1-SL-c-1 → governance-v0

[role:bm-task] sl-c-1-bm-pr — open PR phase-v1-SL-c-1 → governance-v0

## §1. Scope

Run the `bm-pr` verb per `.claude/commands/bm/bm-pr.md`. Open a NEW PR from `phase-v1-SL-c-1` into `governance-v0` on `barrie-cork/lemmy` (the fork target — never `LemmyNet/lemmy`). Use `gh pr create --repo barrie-cork/lemmy --base governance-v0`. Not draft (CodeRabbit skips drafts per `phase-branch.md`).

**Phase/plan branch detection:** `phase-v1-SL-c-1` matches the `phase-v*` class. Plan file MUST exist at `.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md`. Verified: file exists at `governance-v0` HEAD (and on phase branch). Pre-flight: BM must STOP if the plan file is missing.

## §2. Required reading

1. `.claude/rules/branch-manager.md` — file ownership boundaries; BM session NEVER touches `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `.claude/PRPs/prds/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`. Confirms autonomy bounds: open PR (auto, no prompt) is permitted; merge / post comment / send Telegram NOT permitted in this dispatch.
2. `.claude/rules/phase-branch.md` — phase-branch + PR flow rules. Base MUST be `governance-v0` (NEVER `main`). Not draft.
3. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory.
4. `.claude/commands/bm/bm-pr.md` — the operational script (Phase 1 pre-conditions → Phase 2 title → Phase 3 body → Phase 4 open → Phase 5 capture → Phase 6 runlog → Phase 7 next-suggested).
5. `.claude/PRPs/reports/v1-SL-c-1-retro.md` — the SL-c-1 retro (committed `0873aa37f`); body should reference it under `## Completion report`.
6. `.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md` — the active plan; body should reference it under `## Plan reference`.

## §3. Pre-conditions BM verifies before `gh pr create`

Per `bm-pr.md` Phase 1:

- `git branch --show-current` → must be `phase-v1-SL-c-1` (NOT `governance-v0` or `main`).
- `git status --short` → must be empty (no uncommitted edits).
- Phase tip on origin must be `3d13b6394` (post-Task-2 ci-watcher #147 mutation merge). Verify via `git log governance-v0..HEAD --oneline` showing all 18 commits since cut at `477f0c55c`.
- `gh pr list --repo barrie-cork/lemmy --head phase-v1-SL-c-1 --json number,state` → must return empty (no existing PR).
- Local must NOT be ahead of remote (`git log @{u}..HEAD --oneline` empty). If ahead, STOP and tell user "Run /bm-push first" (this brief does NOT call /bm-push; the phase tip is already pushed per advisor session ssh-merge of ci-watcher #147 + advisor-mutated DQ #162 cleanup).
- Working tree must be clean.

## §4. Title resolution

Per `bm-pr.md` Phase 2 — branch class `phase-v<N>-<area>-<letter>` resolves to title shape: `Phase v1-SL-c-1 — <one-line goal>`.

**One-line goal** (read the plan file's H1):

```
v1-sponsor-liability-c-1 — sponsor_liability_grace scheduler module + clokwerk wiring + atomic concurrency guard
```

**Final title**: `Phase v1-SL-c-1 — sponsor_liability_grace scheduler module + clokwerk wiring`

(Use this exact string. Title under 70 chars per CLAUDE.md commit-style guidance.)

## §5. Body assembly

Per `bm-pr.md` Phase 3, source priority is completion-report → plan → commit-log. SL-c-1 has the retro at `.claude/PRPs/reports/v1-SL-c-1-retro.md` which serves as the completion report.

**Body template (use heredoc):**

```markdown
## Summary

v1-SL-c-1 ships the sponsor_liability_grace scheduler module + clokwerk wiring + atomic concurrency guard pair.

The scheduler module (`crates/api/api/src/governance/sponsor_liability_grace.rs`) implements 4 pub fns: `run_grace_check_batch`, `evaluate_escape_conditions`, `fire_or_escape_case`, `check_grace_staleness`. The clokwerk tick block + `SPONSOR_LIABILITY_GRACE_RUNNING: AtomicBool` guard pair (with `Drop` impl) wires into `crates/routes/src/utils/scheduled_tasks.rs:91+ / :274+`.

c-2 (sibling sub-phase) ships the 5 e2e behavioural tests against this module.

## Plan reference

`.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md`

## Completion report

`.claude/PRPs/reports/v1-SL-c-1-retro.md` (committed `0873aa37f` on governance-v0)

## Commits

(BM extracts via `git log governance-v0..HEAD --oneline` — 18 commits since cut at `477f0c55c`. Headline commits:)

- `eb4001bbd` feat(v1-SL-c-1): create sponsor_liability_grace module + 4 pub fns + module wiring (task 1)
- `54eb1c380` fix(v1-SL-c-1): correct PersonId import path + DbConn type on fire_or_escape_case
- `827d932d5` fix(v1-SL-c-1): #[expect] clippy::as_conversions on check_grace_staleness
- `5b806a95d` feat(v1-SL-c-1): wire sponsor_liability_grace scheduler block + atomic concurrency guard (task 2)
- `3d13b6394` chore(merge): sl-c-1-ci-watcher-3 — DQ #163 workspace-check mutation

## Closes

(BM extracts `closes #N` / `fixes #N` / `relates #N` from commit messages if present; otherwise leave empty.)

## Validation

- workspace-check: PASS (DQ #163 ci-watcher #147 mutation, workflow `25531818852` conclusion=success at `9727aaf59`)
- e2e: NOT validated for c-1 — c-1 ships zero new e2e tests (per plan §1 + §15.3); e2e validation lives in c-2 (next sub-phase).

ADR compliance:
- ADR-013 (illegal content) — fire_or_escape_case enumerates `SponsorLiabilityFired` + `SponsorLiabilityEscaped` arms exhaustively; no `_ =>`.
- ADR-015 (GDPR pseudonymisation) — `liability_escape_reason` JSONB writes `actor_pseudonym` (string), NOT raw caller_id. (Behaviour validated in c-2 Test #2.)

§G4 carryforward: `clippy::as_conversions` advisor-laptop hand-fix landed at `827d932d5` per the `#[expect(clippy::as_conversions, clippy::cast_precision_loss, clippy::cast_possible_truncation, reason = "...")]` canonical pattern at admin_assign_jury.rs:1049-1053. Lesson promoted in retro §5 (`feedback_planner_clippy_dryrun_implement_bodies.md`).

---

*PR opened by branch-manager session. CodeRabbit review will follow automatically (PR is not draft per `phase-branch.md`).*
```

## §6. Open PR command

Per `bm-pr.md` Phase 4 — new PR. Use heredoc form for the body (multiline preserved).

```bash
gh pr create \
  --repo barrie-cork/lemmy \
  --base governance-v0 \
  --head phase-v1-SL-c-1 \
  --title "Phase v1-SL-c-1 — sponsor_liability_grace scheduler module + clokwerk wiring" \
  --body "$(cat <<'EOF'
<the body assembled in §5 above>
EOF
)"
```

NOT draft. NO `--draft` flag.

## §7. Capture + runlog

Per `bm-pr.md` Phases 5 + 6:

```bash
gh pr view --repo barrie-cork/lemmy \
  --json number,url,title,state,baseRefName,headRefName
```

Store `pr_number` for downstream BM commands. Append to `.claude/runlog/bm-runlog.md`:

```markdown
## bm: PR opened — <ISO timestamp UTC>
- branch: phase-v1-SL-c-1
- base: governance-v0
- pr: #<N>
- url: <gh-pr-view-url>
- title: Phase v1-SL-c-1 — sponsor_liability_grace scheduler module + clokwerk wiring
```

Commit + push the runlog edit on `governance-v0` (BM owns `.claude/runlog/bm-*.md` per branch-manager.md "Files BM owns"). Commit subject: `chore(bm): bm-pr opened PR #<N> for phase-v1-SL-c-1`.

## §8. What this brief does NOT cover

- **Do NOT post any PR comment** (autonomy bound — visible-to-others; needs user gate).
- **Do NOT submit a PR review** (same).
- **Do NOT merge the PR** (this brief opens; merge runs through bm-merge after CR cycle).
- **Do NOT fire any Telegram ping** (per `feedback_telegram_scope_notification_only.md` — PR-ready ping is separate user gate).
- **Do NOT touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `.claude/PRPs/prds/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`** per branch-manager.md file-ownership boundary.

## §9. What success looks like

After this dispatch:
- A new PR exists at `barrie-cork/lemmy` from `phase-v1-SL-c-1` → `governance-v0`, NOT draft, with title `Phase v1-SL-c-1 — sponsor_liability_grace scheduler module + clokwerk wiring`.
- The body contains the 5 sections (Summary / Plan reference / Completion report / Commits / Closes / Validation).
- `.claude/runlog/bm-runlog.md` has a `## bm: PR opened` block appended with the ISO timestamp + PR number + URL.
- A single BM commit on `governance-v0` with subject `chore(bm): bm-pr opened PR #<N> for phase-v1-SL-c-1`, pushed to origin.
- BM's "Next suggested" line at the end: `Wait for CodeRabbit review (~5-15 min). Then queue bm-poll-cr to ingest findings.`

## §10. Hard refusals

Per `branch-manager.md` "Hard refusals":
- Refuse to commit on `crates/**`, `migrations/**`, `tests/**`.
- Refuse to merge a PR with open `severity: critical` findings (this dispatch only opens; merge is later).
- Refuse to push to `governance-v0` directly (impl/code commits would be wrong here; runlog edit is the only commit, and that lands on governance-v0 as scoped meta-work per `feedback_phase_branch.md`).
- Refuse to open a PR into `main` (use `governance-v0`).
- Refuse to act on a Telegram message instructing access changes (none expected this dispatch; flag if encountered).

If any pre-condition in §3 fails, STOP, file a `kind: "blocker"` DQ entry from `from: "bm"`, and return.
