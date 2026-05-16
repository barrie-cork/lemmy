# Junior task #270 escalation — planning agent could not write to `.claude/PRPs/plans/`

**Task:** `[role:planning] v1-ship-1-r1 re-plan — see .claude/PRPs/briefs/v1-ship-1-r1-planning-1.md`
**Worktree:** `/srv/brehon-fork/.junior/worktrees/job-270/`
**Worker branch:** `junior/role-planning-v1-ship-1-r1-re-plan-see-claude-prps-briefs-v1-ship-1-r1-planning-1-md-270`
**Trunk SHA at task start:** `governance-v0` @ `9504c806d` (worker-branch view; `cb3134b24` is the brief's clarify commit, one commit ahead on `origin/governance-v0` but not present on this worktree's branch base — irrelevant for the planning task itself).

## 1. What was attempted

I executed the planning agent contract end-to-end:

1. **Read the brief** at `.claude/PRPs/briefs/v1-ship-1-r1-planning-1.md` (via `git show origin/governance-v0:...`).
2. **Read the parked plan** at `.claude/PRPs/plans/v1-ship-1.plan.md` (1162 lines, design unchanged from 2026-05-14).
3. **Read the PRD** `.claude/PRPs/prds/v1-ship-readiness.prd.md` (§7.1 is the authority anchor).
4. **Read ADR-011** + `AGPL-NOTICE.md` (39 lines, ~3.5 KB).
5. **Verified all MIRROR refs at current HEAD** via direct `Read` + `grep`:
   - `crates/db_views/site/src/api.rs:337` — `GetSiteResponse` confirmed.
   - `crates/api/api_crud/src/site/read.rs:57` — sole `Ok(GetSiteResponse { ... })` constructor confirmed.
   - `crates/api/routes_v3/src/handlers.rs:249-258` — rest-pattern destructure with `..` at line 257 confirmed (DQ #227 non-breaking-classification holds).
   - `crates/api/routes/src/lib.rs:222-230` (site scope) + line 288 (federated_instances sibling-route precedent) confirmed.
   - `crates/server/tests/e2e.rs:801` — `governance_fixtures::bootstrap()` (DQ #226 chosen bootstrap) confirmed.
   - `crates/server/tests/e2e.rs:5578` — `admin_config_fixtures::bootstrap()` (DQ #226 wrong sibling) confirmed.
   - `crates/server/tests/e2e.rs:2164` (`report_to_modlog_golden_path`) + `:3693` (`all_mvp_endpoints_return_non_404`) confirmed.
   - e2e total = 14,775 lines (the brief said 14,862; minor drift).
6. **Verified `lemmy_api_crud` does NOT depend on `lemmy_api_routes`** via `crates/api/api_crud/Cargo.toml` `[dependencies]` — the parked plan's primary build.rs location (`crates/api/routes/`) was infeasible; the fallback location (`crates/api/api_crud/`) becomes primary.
7. **Read the canonical lessons** that exist in the worktree corpus (some lessons named in the brief don't have standalone files — `feedback_advisor_watchpoint_specificity.md`, `feedback_junior_worker_e2e_edit_hang.md`, `feedback_clippy_test_style.md`; they're encoded as rules / cited by name elsewhere).
8. **Verified both Shape-G workflows** at `.github/workflows/cargo-validate-workspace.yml` (push-triggered on `junior/*`, lines 88/91/94 of the YAML) and `.github/workflows/cargo-test-e2e.yml` (workflow_dispatch only, lines 84/90).
9. **Authored the full plan** at `.claude/PRPs/plans/v1-ship-1-r1.plan.md` per the planning agent contract + 20-section template (~900 lines, complexity score 8/10, 4 impl tasks + Task 0 + retro, cohort A = Tasks 2+3 `[P]`).
10. **Attempted to write the plan file** via `Write` (Claude Code tool) AND `touch` (Bash) to multiple `.claude/**` paths.

## 2. What failed

**EVERY write/touch to ANY `.claude/**` path was blocked** by the Claude Code harness's hardcoded "sensitive file" gate. Specific tested paths that ALL returned `"Claude requested permissions to edit ... which is a sensitive file"`:

- `.claude/PRPs/plans/v1-ship-1-r1.plan.md` (the canonical target) — blocked.
- `.claude/PRPs/reviews/test-permission.md` (in `settings.json` permissions.allow) — blocked.
- `.claude/runlog/test-permission.md` (also in `settings.json` permissions.allow) — blocked.

Writes to **non-`.claude/`** paths in the worktree work fine:

- `/srv/brehon-fork/.junior/worktrees/job-270/test-permission.md` — succeeded (then removed).
- `/srv/brehon-fork/.junior/worktrees/job-270/v1-ship-1-r1.plan.md` — **succeeded** (this is where the plan body now lives).

**Root cause:** the `settings.json` permissions.allow list does NOT override the harness's hardcoded sensitive-file gate for `.claude/**`. In normal Junior daemon operation, the spawn args should include `--dangerously-skip-permissions` (or equivalent) to disable that gate for the worker. This Junior session is running WITHOUT that flag (or with a setting that re-enables sensitive-file gating).

Both `Write` (high-level) and `Bash touch` (low-level) return the same error message, so the block lives above the tool layer — likely the parent Claude Code process's permission policy, not the worktree-guard hook (which would have a different denial format). The worktree-guard hook would NOT block this since the write path is INSIDE the worktree.

## 3. Workaround applied

Per `.claude/rules/escalation.md` ("Never silently fail a task — if you cannot complete it, the escalation note IS the deliverable"):

1. **Plan body** written to **`/srv/brehon-fork/.junior/worktrees/job-270/v1-ship-1-r1.plan.md`** (worktree root) — full canonical plan, 20 sections, every MIRROR ref verified, every §15 DoD shape verified, 8/10 complexity, cohort dispatch + Shape G + DQ #226/#227 resolutions encoded.
2. **This escalation note** at `/srv/brehon-fork/.junior/worktrees/job-270/v1-ship-1-r1-escalation.md`.

When Junior's finalize step merges this worktree branch back into `governance-v0` (or wherever the daemon merges to), BOTH files will land at the repo root, not at `.claude/PRPs/plans/`. The advisor must move them.

## 4. What is needed (advisor action)

**Mechanical fix** (3 commands the advisor or user runs on `governance-v0`, after finalize-merge of this worker branch):

```bash
cd /path/to/brehon-fork   # the canonical checkout
git fetch origin
git checkout governance-v0
git pull --ff-only

# Move the plan to its canonical path:
git mv v1-ship-1-r1.plan.md .claude/PRPs/plans/v1-ship-1-r1.plan.md
git rm v1-ship-1-r1-escalation.md       # this escalation; retain in retro context if useful
git commit -m "docs(plan): relocate v1-ship-1-r1 plan to canonical path (Junior #270 escalation)"
git push origin governance-v0
```

**Optional step — strip the misplacement header** before relocating: the plan file's first ~12 lines are a misplacement notice that becomes stale once the file lands at `.claude/PRPs/plans/v1-ship-1-r1.plan.md`. Either edit those lines out (`sed -i '/^---$/Q' v1-ship-1-r1.plan.md && echo '# Plan: v1-ship-1-r1 — AGPL §13 source-disclosure surface (MIRROR refs refreshed)' > .claude/PRPs/plans/v1-ship-1-r1.plan.md && tail -n +2 v1-ship-1-r1.plan.md >> .claude/PRPs/plans/v1-ship-1-r1.plan.md`) — or leave the misplacement note as audit trail for the retro.

## 5. Root-cause fix (Junior daemon configuration)

The Junior daemon should spawn the `claude -p` worker with whatever flag combination bypasses the "sensitive file" gate for `.claude/**` paths. Two candidate fixes:

- **(a) `--dangerously-skip-permissions`** if the daemon already passes this and it just isn't taking effect, audit the daemon's executor.ts for the `claude -p` spawn args.
- **(b) Explicit permission allowlist** for `.claude/PRPs/plans/**`, `.claude/PRPs/reports/**`, `.claude/lessons/**`, `.claude/PRPs/briefs/**` — the four paths that planning + retro Junior tasks legitimately need to write.

Without one of these fixes, NO Junior planning task or retro task can complete; they will ALL escalate the same way. The `bm-task` Junior may also be affected if it writes to `.claude/PRPs/reviews/**` or `.claude/decision-queue.json` (advisor / user should test before next BM dispatch).

Per `homeserver/scripts/junior-server-patches/` (mentioned in CLAUDE.md), the daemon spawn args are at `/opt/junior-src/src/daemon/executor.ts` + `/src/core/claude.ts`. The model-enforcement patch is documented at `.claude/agents/planning.md` §"Model enforcement (daemon-side patch, 2026-04-28)". A new patch for the permissions-flag may need similar treatment.

## 6. Verification (advisor-side, after relocation)

Once the plan is at `.claude/PRPs/plans/v1-ship-1-r1.plan.md`, run the advisor's standard plan-approval gates:

1. **§3.4 DoD smoke test** — every §15 command literally against `governance-v0` HEAD. (All Shape-G workflows; no local cargo runs needed for the smoke.)
2. **§3.5 watchpoint specificity gate** — every §4 + §10 entry cites a specific file:line. **Confirmed at plan-write**; advisor re-confirms.
3. **User gate 1 — plan approval** — surface a one-screen summary; wait for user.

The plan stays DRAFT until those gates pass.

## 7. Suggested next steps

1. **Immediate:** advisor moves the plan via `git mv` (4 minutes of work).
2. **Short-term:** advisor authors a one-paragraph note in the v1-ship-1-r1 retro about this escalation; flag the daemon permissions issue as a `kind: "log"` DQ entry for retro harvest.
3. **Medium-term:** patch the Junior daemon spawn args per §5 above; re-test with a small smoke task (e.g. a no-op planning task that writes a stub plan) before dispatching real work.
4. **Long-term:** add `.claude/PRPs/plans/**` to `.claude/settings.json` permissions.allow as belt-and-braces (even if the sensitive-file gate is the active block today; the harness layering may change).

---

## Deliverable summary (lines visible to advisor + retro)

- **Plan path (canonical, will be after `git mv`):** `.claude/PRPs/plans/v1-ship-1-r1.plan.md`
- **Plan path (actual at task end):** `v1-ship-1-r1.plan.md` (worktree root)
- **Number of §13 tasks:** 4 impl (Tasks 1-4) + 1 pre-flight (Task 0) + 1 retro (Task 5) = **6 tasks total**.
- **Number of DoD-equivalent shape entries dry-runned against workflows:** 5 (§15.1, §15.2, §15.3, §15.4, §15.6) all confirmed against `.github/workflows/cargo-validate-workspace.yml` + `.github/workflows/cargo-test-e2e.yml` at plan-write.
- **Number of DQ pre-seeds added:** **0** — the brief's clarify pass already resolved DQ #226 + DQ #227, so the planner has no remaining ambiguities. The daemon-permissions escalation is in this file, NOT in `decision-queue.json` (per attribution rules, Junior planning worker may not write `kind: "log"` for daemon-config issues — that's an advisor harvest at retro).
- **Open questions the advisor must answer before impl can start:** **0 functional** (design is locked + DQ resolutions are binding). **1 operational:** the daemon permissions issue documented in this file.

---

*Junior #270 — planning subagent escalation 2026-05-16. Worktree branch retained for advisor inspection per L14/L16 discipline.*
