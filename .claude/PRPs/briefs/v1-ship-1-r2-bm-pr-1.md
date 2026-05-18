# Brief — v1-ship-1-r2 bm-pr (open PR phase-v1-ship-1 → governance-v0)

## 1. Role + dispatch line

`[role:bm-task] bm-pr v1-ship-1-r2 — open PR phase-v1-ship-1 → governance-v0`

Dispatcher → `branch-manager` subagent. Execute `.claude/commands/bm/bm-pr.md` step by step (Phase 1 → 7). Auto, no prompt — `bm-pr` is AUTO-class per the BM autonomy table (PR-create is not a user-confirm action; the merge confirm is a later gate).

## 2. Scope

Open a single PR from `phase-v1-ship-1` into `governance-v0` via `gh pr create --repo barrie-cork/lemmy --base governance-v0`. Resolve title from the plan H1; assemble body from plan reference + commit log + the verify report (no `-complete-report.md` exists — retro is Task 7 post-merge per plan). Capture PR #/URL. Append a runlog entry.

**This sub-phase (v1-ship-1-r2)** delivers the AGPL §13 source-disclosure surface. Tasks 1-4 (DTOs, field+handler+build.rs, /api/v4/source handler+route) are MERGED carry-forward on `phase-v1-ship-1`. Task 6 (commit `38af8b866`) rebuilt the failing acceptance e2e test `agpl_source_disclosure_surface_returns_notice` on the canonical `lib.rs:364` `inner_context` idiom — the R11/DQ #261 type-precision fix that resolves the 5-cycle HTTP 500 (`Data<Data<LemmyContext>>` double-wrap). Phase-2 e2e is GREEN (DQ #263 `result:pass`, 90 passed / 0 failed / 5 ignored pre-existing baseline). `/brehon-verify` ✓ all 3 stories.

Boundaries:
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `Cargo.{toml,lock}`, `rust-toolchain.toml` (BM file-ownership hard refusal).
- Do NOT post a PR comment, submit a review, or send a Telegram ping (those are ASK-first; bm-pr does none of them).
- Do NOT open as `--draft` (CR skips drafts per `phase-branch.md`).
- Do NOT merge — `bm-merge` is a separate later step behind user gate 5.
- Commit ONLY `.claude/runlog/bm-runlog.md` (the Phase 6 append). The PR body is not a committed file.

## 3. Required reading

- `.claude/commands/bm/bm-pr.md` — the operational script (Phases 1-7). Follow literally.
- `.claude/rules/branch-manager.md` — file-ownership boundaries + autonomy table.
- `.claude/rules/phase-branch.md` — PR base = `governance-v0`, not draft.
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` MANDATORY on every `gh pr` call (gh defaults to upstream `LemmyNet/lemmy` on forks).
- `.claude/PRPs/plans/v1-ship-1-r2.plan.md` — read the H1 for the title; §13/§16a for the PR Summary prose.
- `.claude/PRPs/reports/v1-ship-1-r2-verify.md` — reference under PR body (verify ✓ evidence).

## 4. Constraints

- **L14 git-sequence (load-bearing — per PMD #115 + auto-phase.md L14):** the Phase 6 runlog append MUST follow the explicit ordered sequence: **`Edit .claude/runlog/bm-runlog.md` → `git add .claude/runlog/bm-runlog.md` → `git commit -m "chore(bm): ..."` → `git push origin phase-v1-ship-1`** — and ONLY THEN return the Phase 7 output. The BM agent has historically improvised file/git workflow when the brief is silent on sequencing (L11 #148 temp-branch breach, L14 #150 lost-runlog-Edit). Do NOT `git checkout` or `git pull` after the Edit before it is committed+pushed — checkout discards uncommitted worktree Edits.
- **Commit subject:** the runlog commit MUST be `chore(bm): v1-ship-1-r2 PR opened #<N>` (matches BM attribution; NOT `chore(advisor)` — this is a BM-session commit).
- **Phase 1 pre-condition STOPs are real:** if on trunk / branch not pushed / dirty tree / no commits ahead / base resolves to `main` → STOP and surface, do NOT improvise around it. (Pre-checked advisor-side: branch `phase-v1-ship-1` pushed @ `8cdadf9e9`, 107 commits ahead of `governance-v0`, tree clean — all clear.)
- **Phase 1b DQ historical-fail sweep:** runs the `kind=='validate-pending'` filter. Remaining pending is `[229, 248]`; #229 is `kind:log`, #248 is `kind:validate-pending-laptop` (NOT `validate-pending`) — neither matches the sweep filter. Expect "swept 0 entries" (no-op). Do NOT broaden the filter.
- **Phase 1c Phase-2 e2e gate:** the plan touches `crates/server/tests/e2e.rs` so this gate fires. It scans `resolved[]` for a `validate-pending-laptop-e2e` with `result==pass` and `branch` starting `phase-v1-ship-1`. **DQ #263 satisfies this exactly** (already in `resolved[]`, `result:pass`, `branch:phase-v1-ship-1`). Gate PASSES — do NOT re-surface user gate 4 (it was already cleared: user chose "Run local now"; the run is green).
- **Retro gate:** the plan defers retro to Task 7 post-merge (user gate 6). The retro-gate `grep` finds no pre-bm-pr barrier → expect "INFO: plan defers retro to post-merge — no pre-bm-pr retro gate". Do NOT STOP on missing retro.
- **DQ attribution:** if Phase 1b sweeps anything, the commit subject MUST match `^(chore|docs)\((advisor|decision-queue)\)`. BM NEVER writes `answered_by: "advisor"` or `"user"` — only `"bm-self-resolved"` for its own DQ writes (none expected here).
- **Telegram:** do NOT send a `pr-ready` ping (that is an explicit `/bm-ping` ASK-first step, not part of bm-pr).

Brief commit body: this brief authored on `phase-v1-ship-1` (lane worktree). Mandatory file-class lessons: N/A (bm-task, no impl files). §2.3 hybrid search fired: PMD #115 (L14 git-sequence), #335 (BM autonomy/file-ownership), #250 (atomic-raise — N/A here, no ci-watcher). L14 constraint injected into §4 as the load-bearing rule.
