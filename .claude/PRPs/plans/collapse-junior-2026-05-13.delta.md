# Collapse-Junior delta plan (2026-05-13 re-author)

**Base plan:** `C:/Users/barri/.claude/plans/optimized-doodling-storm.md` (2026-05-04, 731 lines)
**Re-author date:** 2026-05-13
**Trigger satisfied:** SL-b shipped 2026-05-05; 4 lanes since (SL-c-1, SL-c-2, SL-d, SL-e); RT-r1 shipped today
**Validation candidate (Phase L):** next sub-phase TBD after Phase K (was SL-c in original plan; SL-c already shipped)

## Verified pre-flight findings (this re-author, vs 2026-05-04 snapshot)

| Field | 2026-05-04 plan | 2026-05-13 reality | Action |
|---|---|---|---|
| HEAD | `e00819e2d` post-SL-a retro | `ca73deec2` post-RT-r1 watch-item 2 | Use current HEAD as baseline |
| DQ pending | 0 | 0 | Same — proceed |
| DQ resolved | 139 | 197 | +58 entries; schema-v2 unchanged |
| `.claude/agents/*.md` | 11 expected | **12** | +1: nothing new beyond plan-expected set; ci-watcher exists per design |
| `.claude/rules/*.md` | 22 expected | **24** | +2 new rules: `auto-phase.md`, `multi-lane-worktree.md` (both Junior-flavored, delete in B.2) |
| `.claude/lessons/*.md` | 105 | **122** | +17 new lessons; ALL PRESERVE per recommendation 4 |
| `.claude/hooks/*.sh` | 11 expected | 10 | One already removed; F.1 delete list unchanged |
| `.claude/commands/handover/` | 2 files | 2 files | B.3 plan still applies |
| User-scope `~/.claude/commands/auto-phase.md` | did not exist | **exists** | Add to J.1 deferred-delete list |
| Working tree | 2 untracked plan files | 1 untracked memory.db | Different file; leave alone |
| Junior daemon | active + enabled | active + enabled | Same — Phase A defer stands |

## Delta to plan B.2 (delete rules)

**Original list (11 files):**
- advisor-orchestrator.md, memory-injection.md, session-awareness.md, post-task-retro.md, evaluation-calibration.md, escalation.md, handover.md, integrator.md, circuit-breaker.md, cross-repo-coordination.md, pmd-search-strategy.md

**Add to delete list (2 new files):**
- **`auto-phase.md`** (376 lines, the orchestration state machine for `/auto-phase` — deeply Junior-coupled; refs advisor session, polling cadence, multi-lane discipline, Junior task IDs)
- **`multi-lane-worktree.md`** (184 lines, multi-lane discipline — Junior daemon worktree pattern; single-session collapse makes this moot)

**Total delete list (13 files).**

## Delta to plan C.4 / B.1 (agents)

**Confirmed inventory (12 agents):**

Keep + lightly edit:
- `branch-manager.md` (pin to Haiku per C.3)
- 7 review/analysis agents: code-reviewer, code-simplifier, comment-analyzer, docs-impact-agent, pr-test-analyzer, silent-failure-hunter, type-design-analyzer (untouched)

Delete:
- `bm-task.md` (per B.1; replaced by kept branch-manager)
- `impl-task.md` (per C.2; replaced by new impl.md)

Rewrite in place:
- `planning.md` (per C.1; drop Junior framing)
- `ci-watcher.md` → CONVERT to `cargo-runner.md` (per C.4)

New file:
- `impl.md` (per C.2; replaces impl-task.md)
- `cargo-runner.md` (per C.4; replaces ci-watcher.md)

**Post-state target:** 10 agents (8 unchanged + planning.md rewritten + impl.md + cargo-runner.md).

## Delta to plan J.1 (user-scope deferred deletes)

**Add `~/.claude/commands/auto-phase.md`** to the deferred-delete list (was not in plan because the command didn't exist in 2026-05-04).

Verify also: `~/.claude/skills/check-dq/`, `~/.claude/skills/brehon-phase-transition/`, `~/.claude/commands/precheck.md`, `~/.claude/commands/start-brehon.md`, `~/.claude/commands/advisor-checkpoint.md` (5 items per plan).

## Delta to plan Phase H (root CLAUDE.md rewrite)

**Current state:** 103 lines (plan target ~70). Existing CLAUDE.md has been rewritten between 2026-05-04 and 2026-05-13 — references current 4-role + auto-phase + multi-lane. Treat as fresh rewrite from spec in plan §H (sections 1-9), not as an edit of current.

## Delta to plan Phase L validation candidate

Original plan: validate via v1-SL-c (the next sub-phase after SL-b ship).
**v1-SL-c has shipped (c-1 + c-2 both merged).** Phase L needs a new candidate. Options:
- v1-JM-f (continue JM lane; natural next slice)
- v1-AD-a (start Admin Dashboard lane; plans b/c exist suggesting AD has some prior work)
- restorative-mechanics-v1 PRD scaffold (start the PRD; requires PRD authorship which itself is a sub-phase)

**Recommendation:** decide Phase L candidate AFTER Phase K passes. Phase K (smoke test) doesn't depend on which sub-phase L uses.

## Execution sequence (this session)

Following the plan's defer-list:

| Phase | Action | Commit subject |
|---|---|---|
| B | Delete 2 agents + 13 rules (incl. auto-phase.md, multi-lane-worktree.md) + 2 handover cmds | `chore(meta): drop Junior dispatch artifacts (2 agents, 13 rules, handover commands)` |
| C | Rewrite planning.md, write impl.md, pin BM to Haiku, convert ci-watcher → cargo-runner | `feat(meta): rewrite planning/impl agents as native CC subagents; pin BM to Haiku; convert ci-watcher to cargo-runner` |
| D | Trim cross-refs in kept rules + commands | `docs(rules): trim Junior/advisor cross-refs from kept rules + commands` |
| E | Rewrite decision-queue.md + .claude/CLAUDE.md cheatsheet | `docs(meta): simplify decision-queue rule + cheatsheet for single-session model` |
| G | Edit `.mcp.json` (no commit — gitignored) | (none) |
| H | Rewrite root CLAUDE.md to ~70 lines | `docs(claude): rewrite CLAUDE.md for single-session subagent model` |
| I | Add `/brehon-flow` command + collapse-lesson | `feat(meta): add /brehon-flow command + collapse-to-single-session lesson` |

Phases A (Junior daemon decommission), F (hook trim), J.1 (user-scope cleanup) all DEFERRED until Phase L passes. Auto-memory J.2 happens after K, before L (per original).

## Gates between phases (per plan)

- **After B:** `ls .claude/agents/` = 10 + 2-to-rewrite = 12 minus 2-deleted = 10 awaiting C; rules count drops to 11 (was 24, delete 13). handover dir gone.
- **After C:** Agents = 10 (8 kept + planning.md + impl.md + cargo-runner.md, no impl-task.md, no ci-watcher.md, no bm-task.md). Grep for Junior/advisor/polling returns 0 hits in agents.
- **After D:** grep across rules + agents + commands for Junior/advisor/polling returns 0 hits (lessons untouched).
- **After E:** decision-queue.md ≤ 200 lines, schema_version=2 holds.
- **After H:** root CLAUDE.md ≤ 80 lines, 0 Junior refs.
- **After I:** `/brehon-flow` readable; new lesson linkable.
- **After K:** parallel-Agent test passes OR records sequential fallback.

## Risks specific to drift

- **The 2 new rules (auto-phase.md, multi-lane-worktree.md) may be referenced by current commands/agents/lessons.** Before deletion, `grep -rl 'auto-phase\\|multi-lane-worktree' .claude/` and decide per hit (lessons stay verbatim per contract; commands/agents/rules get edited).
- **17 new lessons may reference Junior-flavored rules.** They stay verbatim. The MEMORY.md index (Phase J.2) trims to single-session-flavoured but lesson files untouched.
- **`/auto-phase` command body is 700+ lines of state machine.** Its companion file at `~/.claude/commands/auto-phase.md` is user-scope. Deferred-delete per J.1.
- **`v1-RT-r1` retro shipped via `auto-phase` skill.** Retro is on governance-v0; lessons referring to /auto-phase will remain accurate descriptions of how things were done.

## Authorization to proceed

After this delta document is committed, execution proceeds Phase B → C → D → E → G → H → I → K (with parallel-Agent test). Each phase is one commit. Reverts via `git revert <sha>`. Junior daemon stays running through all 7 commits.

This delta document gets committed first as a `docs(meta):` commit so the rewritten files have a clean reference back to the executable spec.
