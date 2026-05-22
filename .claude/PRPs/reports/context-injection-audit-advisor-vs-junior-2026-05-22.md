# Context-injection audit — Advisor (laptop) vs Junior (EliteDesk) — 2026-05-22

**Branch at write:** `2fcda11b7` (`governance-v0`)
**Author:** advisor (laptop, brehon-fork canonical checkout)
**Trigger:** User request, post-memory-prune session — "conduct a comprehensive audit of what context gets injected in at the start of every new session: for advisor on this laptop and for juniors on elitedesk"
**Related prior work:**
- `session-retro-2026-05-09-context-injection-optimisation.md` — first ~9k-token trim (path-scoping 6 rules, CLAUDE.md invariants/state split)
- `session-retro-2026-05-22-context-prune-option-a.md` + `…option-b-2026-05-22.md` handover — this session's predecessor; ~700 tokens shipped, ~14.8k tokens deferred to Option B
- `harness-audit-hooks-2026-05-22.md` — hook directory audit (14 hooks, ~1.8s overhead per Bash call)
- `.claude/lessons/feedback_context_trim_verify_empirically.md` (PMD #147) — empirical-verification-before-trim
- `.claude/lessons/reference_claude_code_rules_loading.md` — empirical loading behaviour CC v2.1.118 (2026-04-23)

## TL;DR

Audited what loads at SessionStart on both surfaces. **Always-loaded rule corpus is ~103k tokens** (51% of 200k window) before either surface does anything — same corpus on both. Junior workers do NOT see laptop-scoped `MEMORY.md` (~6.4k tokens) — that's an advisor-only injection. Junior workers DO see the same 18 always-loaded rules + 9 path-scoped rules from the tracked `.claude/rules/`. The big leverage point for context reduction is the always-loaded rule corpus; MEMORY.md prune work benefits only advisor sessions. **Option B handover identifies the next ~14.8k tokens of savings**, primarily by relocating `auto-phase.md` + `auto-roadmap.md` to `.claude/refs/` (skill-only readers can `Read` on demand). The sentinel-probe gate (now in memory-prune SKILL.md §3.5.c) is mandatory before any relocation.

---

## 1. Topology

| Layer | Advisor (laptop, this session) | Junior workers (EliteDesk daemon) |
|---|---|---|
| Process | Interactive Claude Code session | `claude -p <prompt>` non-interactive, spawned by `junior daemon` per task |
| Model | Opus 4.7 / Sonnet 4.6 (advisor-tier) | Sonnet 4.6 (impl), Haiku 4.5 (BM, ci-watcher), Opus 4.7 (planning) — per `feedback_brehon_subagent_model_effort_assignments.md` |
| CWD | `C:/Users/barri/Developer/brehon-fork` (canonical) OR `brehon-fork-<lane>` (lane worktree) | `/srv/brehon-fork/.junior/worktrees/job-<N>/` (per-task isolated worktree, branched from phase tip) |
| `.claude/` source | Worktree's tracked `.claude/` (same files as daemon copy — branch governed) | Worktree's tracked `.claude/` (cloned from `/srv/brehon-fork` parent at job-spawn) |
| Tool budget | ~150+ tools (full Read/Edit/Write/Bash/Agent/MCP surface) | 5-14 tools per agent contract (pinned narrow per `.claude/agents/<role>.md`) |

Both surfaces see the same 26 rules, 18 skills, 12 agents, 14 hooks — the corpora are intentionally symmetric via the tracked `.claude/` tree.

## 2. What injects at SessionStart (BOTH advisor & Junior)

### A. Always-loaded rules (no `paths:` frontmatter)

| File | Lines | Role |
|---|---|---|
| `CLAUDE.md` | 103 | Project pin: ADRs, four-role model, mandatory gates, BM verb registry |
| `.claude/rules/advisor-orchestrator.md` | 469 | Polling loop, stage shape, cohort dispatch, §G4 classifier, gates |
| `.claude/rules/decision-queue.md` | 558 | DQ schema, kinds, attribution, recipes, hard refusals |
| `.claude/rules/auto-phase.md` | 390 | `/auto-phase` state machine |
| `.claude/rules/auto-roadmap.md` | 399 | `/roadmap-next` + `/auto-roadmap` skill pair |
| `.claude/rules/branch-manager.md` | 265 | BM file ownership, autonomy bounds, session-start ritual |
| `.claude/rules/multi-lane-worktree.md` | 251 | Worktree-per-lane discipline + atomic protocol |
| `.claude/rules/pmd-invariants.md` | 152 | 5 PMD invariants (canonical path, two-systems, no write-time embedding, LESSON trailer, SessionStart guard) |
| `.claude/rules/pmd-search-strategy.md` | 52 | `memory_search_hybrid` vs `memory_search` |
| `.claude/rules/memory-injection.md` | 27 | PATTERNS.md/KNOWN_ISSUES.md + PMD pre-search rule |
| `.claude/rules/phase-branch.md` | 72 | Phase 5+ direct-vs-PR policy |
| `.claude/rules/gh-pr-fork-target.md` | 16 | `--repo barrie-cork/lemmy` mandatory |
| `.claude/rules/post-task-retro.md` | 58 | Stop-hook retro discipline |
| `.claude/rules/integrator.md` | 30 | Input validation |
| `.claude/rules/circuit-breaker.md` | 43 | Retry/fallback caps |
| `.claude/rules/escalation.md` | 47 | Human handoff triggers |
| `.claude/rules/session-awareness.md` | 23 | Concurrent session activity-file claims |
| `.claude/rules/no-destructive-defaults.md` | 7 | No `--force/--no-verify` |

**Subtotal always-loaded:** ~3060 lines / ~103k tokens (50–51% of 200k window).

### B. Conditional / `paths:`-scoped rules (9 — load only on matching Read events)

Per `reference_claude_code_rules_loading.md` (empirical, CC v2.1.118):
- Recursion is on — subdirectories still load
- `paths:` is include-only, no `!pattern` negation
- Loads ONLY on Read events (not Grep, Glob, or Edit)

| Rule | `paths:` scope |
|---|---|
| `cargo-output-capture.md` | `crates/**`, `scripts/brehon/**`, `.github/workflows/**` |
| `no-cargo-output-paste.md` | same as above |
| `cross-repo-coordination.md` | `scripts/**`, `.github/**` |
| `evaluation-calibration.md` | `.claude/PRPs/reports/**`, `.claude/skills/**` |
| `handover.md` | `.claude/PRPs/handovers/**` |
| `pm-plugin-hooks-stable.md` | plugin code paths |
| `view-crate-selectable-template.md` | `crates/**`, `.github/workflows/**` |
| `governance-log-entry-kind-registry.md` | governance code paths |
| `pre-phase-harness-audit.md` | governance code paths |

The mid-session `<system-reminder>` rule injection mechanism works correctly — `handover.md` auto-loaded this session when I edited `.claude/PRPs/handovers/context-prune-option-b-2026-05-22.md`. Per `feedback_context_trim_verify_empirically.md` "Reading the signal" (shipped this session): treat such mid-session rule loads as confirmatory signals about action correctness.

### C. Auto-memory index — **ADVISOR ONLY** (laptop-scoped)

| File | Lines | Role |
|---|---|---|
| `~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/MEMORY.md` | 183 (post-prune 2026-05-22) | User-scope auto-memory index — hard 200-line cap, ~6.4k tokens |
| `~/.claude/CLAUDE.md` | 0 (empty) | User-scope global pin — not in use |
| `~/.claude/projects/.../memory/*.md` (individual memory files) | Variable | One file per memory entry, linked from MEMORY.md |

**Junior workers do NOT see MEMORY.md.** The directory is laptop-local under `~/.claude/projects/`; the daemon's `claude -p` invocation has no equivalent injection path. Junior's only persistence-across-sessions surface is **commit-body `LESSON:` trailers** (advisor harvests at retro time per `feedback_junior_pmd_write_convention.md`).

## 3. SessionStart hooks

**Laptop (per-lane `.claude/settings.local.json` — gitignored, must be re-bootstrapped per lane):**
- `bash .claude/hooks/pmd-canonical-guard.sh` — verifies `.mcp.json` `PROJECT_MEMORY_DB` = canonical absolute path; WARN-not-FAIL on drift
- `bash .claude/hooks/session-start-multi-lane-check.sh` — scans `git worktree list`; WARN if another lane's tip advanced in last 30 min

Per `harness-audit-hooks-2026-05-22.md`: chain costs ~1.4s ceremony per session start. Wired only in gitignored `settings.local.json` — every fresh lane bootstrap re-adds them per `feedback_phase_lane_worktree_bootstrap_checklist.md` step 6.

**EliteDesk daemon (`/srv/brehon-fork/.claude/settings.json` — tracked):**
- `bash ${CLAUDE_PROJECT_DIR}/.claude/hooks/pre-phase-audit.sh` — fires on BOTH `startup` matcher AND `resume` matcher (duplicate wiring; harness-audit-hooks-2026-05-22 Tier-1 #4 recommends collapsing to `startup|resume`)

Each Junior task spawn (`claude -p`) is a new session → both hooks fire per task. The startup-AND-resume duplication is harmless but reads like a configuration mistake — `claude -p` is non-interactive so "resume" semantics never apply.

## 4. MCP servers

**Laptop `.mcp.json`** carries:
- `project-memory` — `PROJECT_MEMORY_DB=C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` (canonical absolute path per `pmd-invariants.md` §1)
- `junior-brehon` — task creation + daemon control (advisor-only)
- `ref-context` — Ref MCP for library docs
- `tavily` — web search

Each MCP server's tool-listing budget consumes context at start.

**EliteDesk Junior worker `.mcp.json` (`/srv/brehon-fork/.mcp.json`):**
- `project-memory` — `PROJECT_MEMORY_DB=/srv/brehon-fork/.project-memory/memory.db` (daemon-local DB, NOT the canonical laptop path)
- `Ref` — HTTP MCP

**Junior workers do NOT have `junior-brehon` MCP** (would be self-recursive). Junior workers do NOT have `tavily`.

**Critical cross-machine PMD divergence:**
- Laptop canonical: `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` (Windows path)
- Daemon: `/srv/brehon-fork/.project-memory/memory.db` (Linux path)

These are **two different DB files on two different machines**. Per `feedback_pmd_split_brehon_vs_homeserver.md` this is intentional (fork=code on laptop+EliteDesk via different filesystems; homeserver-PMD elsewhere is orchestration-scoped) — but cross-machine PMD search is unavailable. Junior `LESSON:` trailer harvest at retro time is the only bridge.

## 5. Skills + Agents

Both surfaces declare 18 skills (`.claude/skills/`) and 12 agents (`.claude/agents/`). **Skills declare via metadata only** (frontmatter `description:`); body loads only on Skill tool invocation or `/<name>` slash. `skillListingBudgetFraction: 0.005` in `settings.json` compresses the per-turn skill index.

**User-scope skills** (`~/.claude/skills/`) — **advisor only**:
- `brehon-phase-transition/SKILL.md`
- `check-dq/SKILL.md`

**User-scope commands** (`~/.claude/commands/`) — **advisor only**:
- `advisor-checkpoint.md`, `auto-phase.md`, `precheck.md`, `roadmap-next.md`, `start-brehon.md`

Junior workers have no access to user-scope skills/commands. All Junior orchestration uses project-scope `.claude/skills/` + `.claude/agents/` only.

## 6. Tool list

| Surface | Tool composition | Token cost |
|---|---|---|
| Advisor | Full Read/Edit/Write/Bash/Glob/Grep/TaskCreate/Agent (12 subagents) + MCP (40+) + Skill + deferred-via-ToolSearch | ~25-30k tokens |
| Junior (impl-task) | Pinned: Read, Edit, Write, Bash, Glob, Grep, LSP, mcp__ref-context__*. ~12 tools | ~5k tokens |
| Junior (bm-task, Haiku 4.5) | Read, Edit, Bash, Glob, Grep. ~7 tools | ~3k tokens |
| Junior (ci-watcher, Haiku 4.5) | Read, Edit, Write, Bash. ~5 tools | ~2k tokens |
| Junior (planning, Opus 4.7) | Read, Glob, Grep, Edit, Write, Bash, Agent, LSP, WebFetch, mcp__ref-context__*. ~14 tools | ~6k tokens |

The tool list itself adds material context.

## 7. Per-spawn context budget (measured this session)

### Advisor session at start
- Always-loaded rules: ~103k tokens
- `MEMORY.md`: ~6.4k (post-prune 2026-05-22)
- `CLAUDE.md`: ~3.6k
- Built-in system prompt + tool definitions: ~25-30k
- Skill listing (skillListingBudgetFraction: 0.005): ~1k
- **Total session-start budget: ~140-150k tokens** (matches the 135.7k / 200k I measured this session's prior leg)
- **Working room: ~50-65k tokens** (25-32% of context)

### Junior session (impl-task) at start
- Always-loaded rules: ~103k (SAME corpus as advisor — topology is symmetric)
- `MEMORY.md`: 0 (does not exist on EliteDesk for Junior)
- `CLAUDE.md`: ~3.6k
- Pinned tool surface: ~5k
- Task description prompt: ~1-3k
- Built-in system prompt: ~25-30k
- **Total Junior session-start budget: ~140-145k tokens**
- **Working room: ~55-60k tokens** (28-30% of context)

Junior starts with ~5-10% MORE working room than advisor (no MEMORY.md, narrower tool surface). Both surfaces are squeezed against the same 200k cap by the always-loaded rule corpus.

## 8. Findings

1. **Junior workers do NOT load MEMORY.md** — laptop-scoped user-auto-memory is structurally invisible to Junior. The only durable Junior→advisor knowledge transfer is the `LESSON:` commit-trailer convention. MEMORY.md prune work this session (~175 tokens saved) **only benefits advisor sessions**; Junior context budget is unaffected.

2. **The Junior `.claude/` is the laptop `.claude/` mirror** — but the **canonical PMD path divergence is real**:
   - Laptop's `.mcp.json` `PROJECT_MEMORY_DB` = Windows canonical path
   - Daemon's `.mcp.json` `PROJECT_MEMORY_DB` = Linux path on daemon
   - These are **two different DB files on two different machines**.
   - Junior `LESSON:` trailer harvest at retro time is the only bridge (sync_lessons → backfill recipe per `post-task-retro` SKILL §5.5).

3. **Daemon SessionStart hook duplicates startup + resume** — both matchers run `pre-phase-audit.sh`. `claude -p` is non-interactive so "resume" semantics never fire; the duplicate is harmless dead weight but reads like a configuration mistake. Worth a one-line cleanup. **Already captured in `harness-audit-hooks-2026-05-22.md` Tier-1 #4.**

4. **Always-loaded corpus is ~103k tokens** (~50% of context window) before either surface does anything. The biggest spenders:
   - `decision-queue.md` (558 lines, ~9k tokens — compressed this session from ~9.7k)
   - `advisor-orchestrator.md` (469 lines, ~17k tokens)
   - `auto-roadmap.md` (399 lines, ~6.5k tokens)
   - `auto-phase.md` (390 lines, ~6k tokens)
   - `branch-manager.md` (265 lines, ~5k tokens)
   - `multi-lane-worktree.md` (251 lines, ~5k tokens)
   
   Per Option B handover, `auto-phase.md` + `auto-roadmap.md` are skill-only readers (the rule docs describe state machines that only the `/auto-phase` + `/auto-roadmap` skills invoke). Could externalise to `.claude/refs/` if sentinel-probe (memory-prune SKILL.md §3.5.c) passes.

5. **`paths:`-scoped rules work well** — 9 rules with frontmatter load only when relevant code paths get Read. The `handover.md` mid-session injection this session was the mechanism working correctly. **Empirically verified CC v2.1.118 (2026-04-23) per `reference_claude_code_rules_loading.md`.**

6. **Daemon `.mcp.json` Ref API key in plaintext** — `/srv/brehon-fork/.mcp.json` contains `x-ref-api-key: ref-aaf8c4813a236811559b`. Worth verifying the file is gitignored and not pushed. (Most likely already gitignored per the canonical pattern — but worth a one-line check.)

7. **The big leverage point for context reduction is the always-loaded rule corpus.** ~103k tokens is structural floor for BOTH surfaces. Every line saved benefits Junior + advisor symmetrically. MEMORY.md prune benefits advisor only (~175 tokens). Option B's auto-phase + auto-roadmap externalisation (~12.5k savings) would benefit BOTH surfaces symmetrically. Tier ordering for next session:
   - **Highest leverage:** Option B Step 2 — relocate `auto-phase.md` + `auto-roadmap.md` (gated on sentinel probe)
   - **Medium leverage:** `advisor-orchestrator.md` §3.7 + §3.8 narrow gates externalisation (Step 5, ~700 tokens)
   - **Medium leverage:** `advisor-orchestrator.md` §6.1 + §6.2 subagent dispatch externalisation (Step 6, ~1.2k tokens — single-occurrence patterns shouldn't be in always-loaded rules per `feedback_principles_not_rules.md`)
   - **Low leverage:** MEMORY.md "Junior daemon" + "Cargo / Rust" trims (Steps 3-4, ~400 tokens combined; advisor-only)

## 9. Historical trajectory

| Date | Action | Savings | Source |
|---|---|---|---|
| 2026-04-23 | First context audit (CC v2.1.118) — empirical loading behavior measurement | n/a (measurement only) | `reference_claude_code_rules_loading.md` |
| 2026-04-23 | Phase A archive trim — `task-hopper.md` moved to `.claude/rules/archived/` | ~0 (subdirectories still load!) | `feedback_context_trim_verify_empirically.md` |
| 2026-04-23 | Phase B fix — archived dir moved OUT of `.claude/rules/`; 6 rules path-scoped; `.claude/CLAUDE.md` deleted; CLAUDE.md prose→YAML | -55% Memory files (34.4k → 15.3k) | `session-retro-2026-05-09-context-injection-optimisation.md` |
| 2026-05-09 | Path-scoping of 6 cargo/governance rules | ~9k tokens | same |
| 2026-05-22 | Memory-prune Option A — MEMORY.md duplicates + `decision-queue.md` §Schema v2 compression | ~700 tokens | `session-retro-2026-05-22-context-prune-option-a.md` |
| 2026-05-22 | Memory-prune Option A second pass (this session) — 7 MEMORY.md rule-redundant cuts | ~175 tokens | this session |
| **DEFERRED** | **Option B Steps 1-6** (next session, gated on sentinel probe) | **~14.8k tokens** | `.claude/PRPs/handovers/context-prune-option-b-2026-05-22.md` |
| **DEFERRED** | **Option C** (aggressive `advisor-orchestrator.md` 17k → skeleton + refs) | **~10-12k tokens** (speculative) | same handover, "What NOT to do in next session" |

**Cumulative trajectory:** ~9k (2026-05-09) + ~875 (2026-05-22) = ~10k tokens saved over ~2 weeks. Option B if executed would more than 2× that.

## 10. Recommendations

### Tier 1 — high leverage, low cost

#### 1. **Execute Option B Step 1 (sentinel probe) in a fresh session.** Hard gate before any relocation work. Procedure now codified in `.claude/skills/memory-prune/SKILL.md` §3.5.c. Run the sentinel-probe + report PASS/FAIL.

#### 2. **If sentinel probe PASSES, execute Option B Steps 2-6.** Expected savings ~14.8k tokens, benefits BOTH advisor and Junior symmetrically. Plan is durably captured in the Option B handover.

#### 3. **Consolidate the duplicate `pre-phase-audit.sh` SessionStart wiring on the daemon.** One-line fix: `"matcher": "startup|resume"`. Already captured in `harness-audit-hooks-2026-05-22.md` Tier-1 #4.

### Tier 2 — medium cost, real impact

#### 4. **Re-verify `paths:`-scoped rule behaviour against current CC version.** The 2026-04-23 measurement was on CC v2.1.118. The `feedback_context_trim_verify_empirically.md` lesson explicitly flags re-verification after major CC version upgrades. Mechanical: create `.claude/rules/_test_trigger.md` with a unique sentinel + `paths:` scope, fresh session, Read a non-matching file, check `/context` for absence of the trigger.

#### 5. **Reconsider `feedback_principles_not_rules.md` doctrine for §6.1 + §6.2 of `advisor-orchestrator.md`.** Both have explicit `Promotion status:` fields naming themselves as `defer-pending-2nd-recurrence` / `record-only, single occurrence`. By the doctrine these shouldn't be in always-loaded rules. Externalise both to `.claude/refs/advisor-subagent-dispatch.md`, leave one-line pointer. Keep §6.3 invariants inline.

### Tier 3 — defer (speculative)

#### 6. **`advisor-orchestrator.md` 17k → skeleton + on-demand refs (Option C).** Aggressive structural reshape. Defer until either (a) sessions regularly hit autocompact, (b) Read-on-demand of procedure refs becomes the polling-cadence bottleneck, OR (c) a retro flags "advisor missed routing detail X" ≥2× in distinct sub-phases. Current state: ~25-32% working room at session start — uncomfortable but not red.

#### 7. **Cross-machine PMD bridge.** Currently `LESSON:` trailer harvest at retro time. Alternatives:
   - Periodic daemon → laptop PMD sync (cron — but the canonical-path invariant means content has to be normalised)
   - Junior writes to canonical-path PMD via Tailscale-routed Ollama (already exists for embeddings; could extend to writes)
   - Per `pmd-invariants.md` §3, "no write-time embedding" means even Junior PMD writes would need a backfill leg
   - Speculative; defer pending need

## 11. What this audit does NOT change

- No file edits this session beyond the report itself.
- No `git commit` for hook/rule changes — recommendations only.
- No sentinel probe run this session (Option B Step 1 requires a fresh session per `feedback_context_trim_verify_empirically.md` `paths:`-on-Read constraint).

The user reviews this report and decides which (if any) Tier-1 recommendations to action in subsequent sessions.

## See also

- `.claude/PRPs/handovers/context-prune-option-b-2026-05-22.md` — the canonical execution plan
- `.claude/PRPs/reports/session-retro-2026-05-09-context-injection-optimisation.md` — first context audit
- `.claude/PRPs/reports/session-retro-2026-05-22-context-prune-option-a.md` + `…memory-prune-gate-ship.md` — predecessor sessions
- `.claude/PRPs/reports/harness-audit-hooks-2026-05-22.md` — hook directory audit
- `.claude/lessons/feedback_context_trim_verify_empirically.md` (PMD #147) — empirical-verification-before-trim
- `.claude/lessons/reference_claude_code_rules_loading.md` — empirical loading behavior CC v2.1.118
- `.claude/lessons/feedback_pmd_split_brehon_vs_homeserver.md` — cross-machine PMD intent
- `.claude/lessons/feedback_pmd_cross_lane_canonical_db.md` — canonical-path invariant
- `.claude/lessons/feedback_junior_pmd_write_convention.md` — LESSON-trailer harvest pattern
- `.claude/rules/pmd-invariants.md` §1, §3, §5 — canonical path, no write-time embedding, SessionStart guard
- `.claude/skills/memory-prune/SKILL.md` §3.5 — rule-side cut gate (shipped 2026-05-22)
