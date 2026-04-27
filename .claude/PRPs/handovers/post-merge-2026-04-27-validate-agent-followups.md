# Handover — v1-validate-agent post-merge follow-ups (3 sibling-repo edits + advisor flag)

**Author:** foreground impl session, brehon-fork CWD, on `governance-v0` post-merge
**When:** 2026-04-27, after merge commit `ed4be2bd8` landed at 21:49:34Z
**Mode:** advisory — no code changes happen in brehon-fork; this brief lists what needs doing in `homeserver/` and via the advisor session
**Audience:** the next session that opens `homeserver/` (Claude Code or user-direct), AND the persistent advisor session on its next polling tick

## Bootstrap prompt (paste into next session opening homeserver/)

> v1-validate-agent shipped today (PR #104 → merge commit `barrie-cork/lemmy@ed4be2bd8`). Three post-merge follow-ups live in `homeserver/` (sibling repo to brehon-fork). Read `.claude/PRPs/handovers/post-merge-2026-04-27-validate-agent-followups.md` (this file, in brehon-fork on `governance-v0` @ ed4be2bd8) end-to-end, then execute the three items in §"What needs doing." Each is a small targeted edit; total <30 min.

## Why this brief exists

Per `feedback_library_add_after_shipping.md`, every new file in `crates/`, `.github/workflows/`, `.claude/agents/`, or `.claude/PRPs/templates/` must be registered in `homeserver/library.yaml` so it appears in `/library list/search/sync`. Per the v1-validate-agent plan §17 Completion checklist, this registration is explicitly **out-of-band** — performed in the homeserver checkout, not as part of the v1-validate-agent PR. The advisor flag (item 3) is similarly out-of-band — it lives in advisor-session memory + DQ, not in any committed brehon-fork file.

This handover brief makes the three out-of-band items concrete + actionable so a fresh session in homeserver doesn't need to reverse-engineer them from the v1-validate-agent retro.

## What needs doing

### 1. `homeserver/library.yaml` — register 4 new artefacts

**Why:** New files invisible to `/library list/search/sync` until registered. Drift here means the homeserver session can't find ci-watcher when looking up Junior subagent docs by /library command.

**Path:** `homeserver/library.yaml`

**Files to add (4 entries):**

| Brehon-fork path | Suggested library.yaml description |
|---|---|
| `.github/workflows/cargo-validate-workspace.yml` | "Out-of-band cargo check + clippy + test --no-run on push to phase-v1-* / junior/* (Shape G)" |
| `.github/workflows/cargo-validate-migration.yml` | "Out-of-band migration round-trip on push touching migrations/** (Shape G; stub until v1-JM-e ships real round-trip)" |
| `.claude/agents/ci-watcher.md` | "Fifth Junior subagent — Haiku 4.5, low effort, narrow tools — polls workflow runs via gh run watch + gh run view conclusion; writes validate-result/validate-failed DQ entries" |
| `.claude/PRPs/templates/ci-watcher-brief.template.md` | "Brief template for ci-watcher subagent dispatch — three fields (workflow_run_id / branch / phase_task) + classifier sequence" |

**Sample diff shape** (verify against the actual library.yaml schema before applying — the path field key may differ):

```yaml
# Append to the relevant sections of library.yaml; preserve existing entries
- path: brehon-fork/.github/workflows/cargo-validate-workspace.yml
  kind: workflow
  description: "Out-of-band cargo check + clippy + test --no-run on push to phase-v1-* / junior/* (Shape G)"
  added: 2026-04-27

- path: brehon-fork/.github/workflows/cargo-validate-migration.yml
  kind: workflow
  description: "Out-of-band migration round-trip on push touching migrations/** (Shape G; stub until v1-JM-e ships real round-trip)"
  added: 2026-04-27

- path: brehon-fork/.claude/agents/ci-watcher.md
  kind: agent
  description: "Fifth Junior subagent — Haiku 4.5, low effort, narrow tools — polls workflow runs via gh run watch + gh run view conclusion; writes validate-result/validate-failed DQ entries"
  added: 2026-04-27

- path: brehon-fork/.claude/PRPs/templates/ci-watcher-brief.template.md
  kind: template
  description: "Brief template for ci-watcher subagent dispatch — three fields (workflow_run_id / branch / phase_task) + classifier sequence"
  added: 2026-04-27
```

**Validation:** after committing, run `/library list` (or whatever the homeserver-side command is) and confirm the 4 new entries appear.

**Commit subject suggestion:** `chore(library): register v1-validate-agent artefacts (workflows + ci-watcher subagent + brief template)`

### 2. `homeserver/CLAUDE.md` — ci-watcher-is-Junior-subagent note

**Why:** Per plan §11 Files to change "homeserver/CLAUDE.md — one-line note that ci-watcher is a Junior subagent (not a daemon) and that workflow runs are GH-Actions-side." Some advisor-orchestrator references describe ci-watcher in passing; the homeserver CLAUDE.md is the canonical place to disambiguate "ci-watcher is a subagent, not a daemon you start with systemctl."

**Path:** `homeserver/CLAUDE.md`

**Content to add:** one line in the existing "Junior subagents" or equivalent section. Suggested wording:

> ci-watcher (Haiku 4.5, low effort) is a **Junior subagent** dispatched from `[role:ci-watcher]` task descriptions, NOT a long-running daemon. Workflow validation runs are GH-Actions-side under Shape G (per `brehon-fork/.claude/PRPs/plans/v1-validate-agent.plan.md`); ci-watcher is the polling adapter, not the validator.

If `homeserver/CLAUDE.md` doesn't already enumerate Junior subagents by name, place this line under whatever section first introduces planning / impl-task / bm-task. The goal is disambiguation: the next homeserver session that reads CLAUDE.md should NOT think ci-watcher needs starting with `systemctl --user start`.

**Commit subject suggestion:** `docs(claude-md): ci-watcher is fifth Junior subagent (Shape G), not a daemon`

### 3. Advisor flag — JM-d Task 2 is now unblocked per DQ #61

**Why:** v1-JM-d Task 2 was parked per DQ #61 (advisor 2026-04-27 plan-mode, when v1-validate-agent was being scoped) until validate-agent's bm-merge landed. Merge happened at 2026-04-27 21:49:34Z — the unblock condition is satisfied. The advisor's polling loop on its next tick can now legitimately dispatch JM-d Task 2 against the now-Shape-G-compliant `jm-d-impl-2.md` brief.

**Where:** advisor session memory + (optional) DQ entry annotation.

**Concrete actions for the advisor session:**

1. On next polling tick after `git fetch origin`, the advisor should observe `governance-v0` advanced to `ed4be2bd8` (merge commit). The 14 phase-v1-validate-agent commits land at this point.
2. Read DQ #61 (now resolved). The original answer says JM-d Task 2 queueable after validate-agent ships. Bm-merge has shipped.
3. Optionally append a `kind: log` entry from advisor noting the unblock: `chore(advisor): log validate-agent merge — JM-d Task 2 now queueable per DQ #61`.
4. Stage v1-JM-e or v1-JM-d Task 2 dispatch per the existing JM roadmap (whichever is sequenced first per the homeserver advisor's stage-shape map).

**No edits needed in brehon-fork for this item — it's purely advisor-side recognition.**

The persistent advisor session lives at the laptop; this brief is for the laptop session's next reading. If the advisor is already running and polling, it'll naturally pick this up on next `git fetch`. If a fresh advisor session starts, the bootstrap prompt at top of this file primes it.

## Cross-checks

Before doing the three items, verify nothing has shifted underneath:

```bash
# 1. v1-validate-agent merge is on origin/governance-v0
git fetch origin
git log origin/governance-v0 --oneline -3 | grep -q ed4be2bd8 && echo "OK: merge present"

# 2. Phase branch retained (per phase-branch.md)
git branch -r | grep -q origin/phase-v1-validate-agent && echo "OK: phase branch retained"

# 3. The 4 new artefacts exist on governance-v0
for f in \
  .github/workflows/cargo-validate-workspace.yml \
  .github/workflows/cargo-validate-migration.yml \
  .claude/agents/ci-watcher.md \
  .claude/PRPs/templates/ci-watcher-brief.template.md; do
  git ls-tree -r origin/governance-v0 --name-only | grep -q "^$f$" && echo "OK: $f"
done
```

Expected: 6 OK lines.

## What's NOT in scope for this handover

- **Re-running anything in brehon-fork.** PR #104 merged clean. Workflow runs `25020638716` + `25021266534` were green. The retro at `.claude/PRPs/reports/v1-validate-agent-retro.md` and the verify report at `.claude/PRPs/reports/v1-validate-agent-verify.md` are committed and reachable from `governance-v0`.
- **Branch-protection rule configuration.** DQ #66 deferred this — GitHub-UI action requiring `cargo-validate-workspace.yml` to pass before merge into `governance-v0`. Tracked as a v1-JM-e watch-item per plan §20 + retro §5.3.
- **PMD ingest pipeline gap.** Three brief-named lessons not surfaced by `memory_search_hybrid` during plan-write. Homeserver-side diagnostic, NOT brehon-fork; tracked at retro §5.5.
- **Cancelled / queued empirical probes for ci-watcher.** Deferred — conclusion-string fallback covers all eight enumerable conclusion values, so the deferred probes are non-load-bearing.
- **§G4 classifier allowlist tuning.** Empirical evidence required (Stories 4 + 5 deferred); v1-JM-e or v1-rep-tuning-r3 retro proposes additions/removals.

## See also (for the next session)

- `brehon-fork/.claude/PRPs/plans/v1-validate-agent.plan.md` — original plan (confidence 7/10)
- `brehon-fork/.claude/PRPs/reports/v1-validate-agent-retro.md` — retro (confidence 8/10 with empirical-evidence boost)
- `brehon-fork/.claude/PRPs/reports/v1-validate-agent-verify.md` — /brehon-verify report (all stories ✓ or [deferred-to-retro] per design)
- `brehon-fork/.claude/PRPs/reviews/pr-104-findings.yaml` (gitignored — only on local machine where the BM session ran) — CR triage matrix, 9/9 disposed (7 done + 1 rebut + 1 fix-in-pr→done)
- `brehon-fork/.claude/lessons/feedback_gh_run_watch_exit_status_unreliable.md` — load-bearing lesson; gh CLI 2.89.0 false-zero bug; ci-watcher classifies via `gh run view --json conclusion`, NOT exit code
- `brehon-fork/.claude/lessons/feedback_dtolnay_rust_toolchain_components_with_toml.md` — toolchain action ignores `components:` when rust-toolchain.toml present; derive from `rustup show active-toolchain`
- `brehon-fork/.claude/lessons/feedback_planner_dod_dry_run_caught_partial.md` — meta-lesson; §15 dry-run catches YAML parse but not nested-shell semantic correctness; first-push validation is the real gate

---

_Handover authored by foreground impl session that shipped v1-validate-agent end-to-end (foreground, NOT four-role Junior — see retro §2 framing). The session can sign off after committing this brief; advisor session picks up from polling-loop tick onwards._
