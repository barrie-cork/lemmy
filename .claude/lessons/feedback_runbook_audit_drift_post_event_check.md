---
name: Runbook / audit / plan claims drift from reality — verify each claim against live state before executing
description: Runbooks, migration audits, AND plan bodies written before an operation commonly mis-state current reality — both LOCAL facts (already-gitignored files, wrong counts, inverted polarity, same-name-different-content files) and EXTERNAL facts (stale Dependabot alert counts, closed/wrong upstream PRs, which-version-is-the-fix inversions). Treat every such claim as a hypothesis to verify, not a fact to execute. External facts drift faster and are higher-stakes.
type: feedback
---

When a runbook, pre-migration audit, OR plan body asserts a fact about current state — "file X is not gitignored", "there are N servers", "file A and file B are duplicates", "there are 13 open alerts", "upstream PR #N is open", "version V is the fix release" — verify each claim against the actual live state before acting on it. This applies to **two classes** of fact, and the second is the more dangerous:

- **LOCAL state** (repo files, counts, gitignore membership, file identity) — drifts when a prior session edits the repo between plan-write and plan-execute.
- **EXTERNAL state** (Dependabot alert counts/severities, upstream PR/release numbers + open-closed state, published crate versions, which-version-carries-the-fix) — drifts *independently of the repo*, so it goes stale faster, and a wrong external fact tends to be higher-stakes (a dismissal built on "this version is the fix" can be a false security claim; a watch set on a closed PR silently never fires).

**Why:** The advisor-CWD migration (2026-04-30) audited the brehon-fork and homeserver CWDs before migration. The audit got several material details wrong at execution time:

- `.mcp.json` was already in `.gitignore` (line 81 — the audit said it was not)
- `.mcp.json` had 6 configured MCP servers, not 8 (the audit counted stubs and dev entries)
- `memory-injection.md` polarity was inverted — brehon-fork PMD was the richer system, not homeserver (the audit said the opposite)
- The three "overlap files" (rules that appeared in both repos) had diverged content — they were not identical copies. Treating them as "pick one" would have lost changes.
- Commands listed as needing relative-path patches had no relative paths to patch (already used correct paths)

None of these errors blocked the migration outright, but each required a live re-check under smoke pressure, adding friction and risk of error compounding.

**Why (external-state axis, added 2026-06-04 — v1-closeout Phases 2+5):** the same drift class hit a *plan body* (not just a runbook), and on EXTERNAL facts. A v1-closeout plan authored the same session-day stated facts that were stale **5 separate times** in two ~plan-sections — every one load-bearing, every one caught by a pre-action live verify:

- "RT-r2 retro path is a placeholder; annotate if a dedicated retro is authored" → `ls` showed `v1-RT-r2-retro.md` already existed → correct fix was **repoint**, not annotate. (file-existence — local, but the *plan's instruction* was conditioned on a stale check.)
- "13 aarch64-only wasmtime criticals" → `gh api .../dependabot/alerts` showed **12** alerts, only 2 criticals aarch64-only, ~10 medium/low incl. one explicitly x86_64. (alert count + exposure-polarity — EXTERNAL.)
- "watch extism PR #847 (open since 2026-04-07)" → `WebFetch` showed #847 **CLOSED** 2025-05-19, wrong-direction; extism already shipped its wasmtime-41 bump in v1.21.0 (our pin) via a *different* PR. Setting a watch on #847 would have watched a dead PR forever. (upstream PR state — EXTERNAL.)
- "set a `/schedule` watch" → the watch horizon is months but `/schedule`/`CronCreate` auto-expire at 7 days → silent lapse (see `feedback_schedule_cron_expiry_vs_watch_horizon.md`).
- "astral-tokio-tar 0.6.0 IS the fix release; dispute it" → lockfile + `gh api` showed we pin the **vulnerable** 0.6.0; 0.6.1/0.6.2 are the fixes. Dismissing-as-disputed would have been a **false security claim**. (which-version-is-the-fix inversion — EXTERNAL, highest-stakes of the five.)

The lesson: when a plan asserts an external fact, the plan author captured it at write time and the world moved past it. The longer-lived the plan (close-out plans span weeks across milestone boundaries), the staler its external facts. Verify in the same step you act.

**How to apply:**

- For any runbook/plan step that makes a factual claim about LOCAL state ("file X exists/is-gitignored/has-N-lines/is-identical-to-Y"), add an explicit verify step before the action step:
  - `.gitignore` membership: `grep -n '<file>' .gitignore` before concluding it is or isn't tracked
  - Count claims: `wc -l <file>` or `ls -1 | wc -l` before asserting a count
  - "Identical file" claims: `diff <a> <b>` before treating as duplicates — same filename ≠ same content after parallel editing
  - Polarity / which-system-is-richer claims: read both sides and compare line counts or section headers before deciding which is canonical
  - File-existence-conditioned instructions ("annotate IF a dedicated X is authored"): `ls`/`glob` for X first — the condition may already be met, flipping the action.
- For any step that makes a claim about EXTERNAL state, verify with the live API in the **same step** before acting (these drift independently of the repo, so re-verify even if the plan is hours old):
  - Dependabot alert count/severity: `gh api repos/<o>/<r>/dependabot/alerts --paginate --jq '[.[]|select(.state=="open")]|length'` — never trust a count in prose.
  - "version V is the fix release": check `.security_vulnerability.first_patched_version.identifier` per alert AND the version in `Cargo.lock` — confirm V is actually ≥ the patched version, not equal-to or below it. (The T3 inversion: 0.6.0 was *below* the 0.6.1 patch.)
  - Upstream PR/release state: `WebFetch` the PR/releases URL or `curl crates.io/api/v1/crates/<c>` — confirm open/closed + direction + that it's the PR you think it is (numbers get misremembered).
  - Which dependency tree a vuln is in (prod vs dev): trace the consumer in `Cargo.lock` (`grep -B<n> 'name = "<crate>"'`) — a dev-only dep (test harness) has different exposure than a prod dep.
- Write runbooks with a `## Pre-flight verification` block that lists the live checks needed to validate the audit's assumptions, separate from the `## Steps` block that executes.
- If a runbook claim is wrong, update the runbook immediately (same session) so the next reader starts from reality.

**Symptom to recognise:** You start executing a runbook/plan step and the action doesn't make sense (e.g. "add X to .gitignore" but `grep` shows it's already there; "dispute version V as the fix" but the alert says V is below the patched version). That's a drift event — stop, verify the remaining claims in the same section, then continue. **For external facts specifically:** if a plan section cites a count, an upstream PR/release number, or "version V is the fix", treat that as the trigger to run the live API check *before* the first action that depends on it.

**Generalises to:** Any pre-written audit, migration checklist, multi-step setup doc, OR plan body authored before execution. The longer the gap between writing and running, the more drift accumulates — and EXTERNAL facts (upstream PRs/releases, dependency-advisory counts, which-version-is-the-fix) drift on the *upstream's* clock, not the repo's, so a plan that's "fresh" by repo-commit time can still cite a months-stale external fact. Post-event verification is mandatory if the gap was > 1 day for local facts, and **always** for external facts regardless of plan age.

**Related lessons:**
- `feedback_plan_drift_metadata_cross_check.md` — same class (plan claim vs reality diverges between write and execute time)
- `feedback_test_scripts_against_real_targets.md` — test scripts against real input, not mental models
- `feedback_verify_automated_reviewer_claims_against_compiler.md` — sibling: CR/Copilot trait/type claims are hypotheses, compile-check before acting (same "external assertion ≠ fact" discipline)
- `feedback_schedule_cron_expiry_vs_watch_horizon.md` — the `/schedule` 7-day-expiry footgun surfaced in the same v1-closeout session
- PMD eval id 785 — the v1-closeout Phases 2+5 evidence (5× recurrence); session retro `session-retro-2026-06-04-closeout-phases-2-and-5.md`
