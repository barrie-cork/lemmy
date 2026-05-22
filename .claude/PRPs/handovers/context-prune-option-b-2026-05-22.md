# Context-prune Option B — handover for next session

> **Authored 2026-05-22 by advisor session on `governance-v0`.** Self-contained:
> readable with zero conversation context.

## What this is

Plan for the **next session** to execute Option B of the context-injection
prune. Option A landed this session (commits TBD; MEMORY.md trimmed +
`decision-queue.md` §Schema compressed). Option B targets the rule
relocations and the rest of the MEMORY.md cuts identified during the
session's `/context` audit.

## Where we are now

### Option A — DONE this session

- MEMORY.md: 200 → 187 lines. Removed 10 duplicates with auto-loaded
  rules (four-role model, autonomy goals, prp-spec-kit, phase-transition
  skill ref, nutomic tools, DQ hook informational, e2e local-vs-dispatch,
  retro-not-report, four-role-retro-signals, parallel-agents-one-worktree).
  All recorded in MEMORY.md "Historical" section line 187.
- `.claude/rules/decision-queue.md`: 563 → 558 lines. §"Schema (v2)"
  history block compressed into §"Schema" current-state block (~300
  tokens). v3 back-compat rule preserved verbatim.

### Baseline /context measurement at session end

- 135.7k / 200k tokens (68%)
- Memory files: 65.1k tokens (32.6%) — largest single category
- Top 3 memory files: `advisor-orchestrator.md` (17k), `decision-queue.md`
  (~9.4k post-compression), `auto-roadmap.md` (6.5k), `auto-phase.md`
  (6k), `MEMORY.md` (~7.4k post-prune)

### Total Option A savings: ~700 tokens

Below the projected 3.5–4k because two planned rule-side cuts were
correctly rejected on second-look:
- `advisor-orchestrator.md` §G4 example block — externalizing it would
  defeat the anti-paraphrase gate; the verbatim example IS the gate
- `decision-queue.md` §"How to write a question" — Junior workers
  (impl/bm) read it at task dispatch; externalizing pushes the cost to
  every Junior boot

## Option B scope — what to do next session

**Target:** ~13k tokens saved (Memory files 65k → ~52k). Brings total
context from 68% → ~62%.

**Posture:** "the harness is working well" (user, 2026-05-22) — don't
break what's load-bearing. Cut what's command-scoped or load-on-demand.

### Step 1 — Sentinel-probe `.claude/refs/` (MANDATORY first action)

Per `feedback_context_trim_verify_empirically.md` (PMD #147): every
context-trim that depends on load behavior MUST be verified
empirically in a fresh session BEFORE the actual moves. Documentation
claims about what loads have been wrong before — Phase A's archive
trim into `.claude/rules/archived/` shipped ~0 savings until proven
in a fresh-session `/context` measurement.

**Procedure:**

1. Create `.claude/refs/_test_trigger.md` with a unique sentinel string
   (e.g. `SENTINEL_REFS_TEST_20260523_ABCD1234`).
2. Commit + push to `governance-v0`.
3. Close current Claude Code session.
4. Open a fresh session in `C:/Users/barri/Developer/brehon-fork`.
5. Run `/context`. Inspect the "Memory files" list.
6. **PASS:** the test trigger file is NOT in the Memory files list.
   `.claude/refs/` does not auto-load. Proceed to Step 2.
7. **FAIL:** the test trigger file IS in the Memory files list.
   `.claude/refs/` auto-loads recursively. **STOP** — revert plan,
   re-scope Option B around in-rule compression rather than
   relocation.
8. After PASS, delete `.claude/refs/_test_trigger.md` + commit.

### Step 2 — Move `auto-phase.md` and `auto-roadmap.md` out of `.claude/rules/`

**Only if Step 1 PASSED.**

- `git mv .claude/rules/auto-phase.md .claude/refs/auto-phase.md`
- `git mv .claude/rules/auto-roadmap.md .claude/refs/auto-roadmap.md`
- Edit `~/.claude/commands/auto-phase.md` (user-scope skill body):
  add a `Read .claude/refs/auto-phase.md` step in Phase 0 (before any
  state-routing decision)
- Edit `~/.claude/commands/auto-roadmap.md` + `~/.claude/commands/roadmap-next.md`:
  same — add `Read .claude/refs/auto-roadmap.md` in their respective
  Phase 0
- Edit `.claude/rules/branch-manager.md` and `.claude/rules/multi-lane-worktree.md`
  "See also" / cross-references that name `auto-phase.md` /
  `auto-roadmap.md`: update paths to `.claude/refs/`
- Edit `CLAUDE.md` "Where to look next" sub-section if it names
  these rule files by path

**Verify after move:** fresh session → `/context` → confirm
auto-phase.md and auto-roadmap.md no longer appear in Memory files;
confirm `MEMORY.md`, `advisor-orchestrator.md`, `decision-queue.md`
unchanged.

**Expected savings:** ~12.5k tokens (6k + 6.5k).

### Step 3 — MEMORY.md "Junior daemon" section trims

Move to Historical (kept in PMD; not auto-loaded):

- `feedback_junior_finalize_stage_per_job` — operational texture
- `feedback_junior_finalize_skips_when_worker_pre_pushes` — same
- `feedback_junior_cancel_db_lock_zombie` — recovery recipe, low frequency
- `feedback_junior_task_liveness_check` — diagnostic

**Expected savings:** ~6 lines / ~250 tokens.

### Step 4 — MEMORY.md "Cargo / Rust" section trims

Move to Historical:

- `feedback_e2e_filter_assumes_naming`
- `feedback_rate_limit_debug_config_post_bucket`
- `feedback_pin_strategy_check_derives`

**Why:** these are impl-task reference cards. The impl-task subagent
loads `.claude/lessons/` at task start (per file-class injection
table in `advisor-orchestrator.md` §2.4). The advisor session almost
never compiles.

**Expected savings:** ~3 lines / ~150 tokens.

### Step 5 — `advisor-orchestrator.md` §3.7 + §3.8 externalization

The two gates fire only when:
- §3.7 Dogfood gate: authoring a NEW `.claude/commands/<verb>.md` slash
  command — extremely low frequency (~1× per quarter)
- §3.8 Schema-changing-spec retrofit gate: plan-mode produces a new
  artifact shape — extremely low frequency

**Move to:** `.claude/refs/advisor-narrow-gates.md` (new file,
`.claude/refs/` confirmed non-loading by Step 1).

**Replace in rule with:** one-line pointer:
```
### 3.7 Dogfood gate + 3.8 Schema-retrofit gate

Both gates fire only when authoring new slash commands or when plan-mode
produces a new artifact shape. Procedure: `.claude/refs/advisor-narrow-gates.md`.
```

**Expected savings:** ~25 lines from `advisor-orchestrator.md` /
~700 tokens.

### Step 6 — `advisor-orchestrator.md` §6 "Subagent delegation" externalization

§6 has three sub-sections (§6.1, §6.2, §6.3). Each carries an explicit
"Promotion status:" field:
- §6.1 status: `defer-pending-2nd-recurrence`
- §6.2 status: `record-only, single occurrence`
- §6.3 invariants (always-applicable subset)

Per `feedback_principles_not_rules.md` doctrine (single-occurrence
patterns shouldn't be in always-loaded rules):

- Move §6.1 + §6.2 + their introductions to
  `.claude/refs/advisor-subagent-dispatch.md`
- **Keep §6.3 inline** (two invariants — brief sub-agents like
  smart colleague; trust but verify — apply always, are short, and
  pair with §6.2 grep verification once §6.2 promotes)
- Replace §6.1 + §6.2 with one-line pointer

When the second recurrence happens for §6.1 or §6.2, lift back
into the rule.

**Expected savings:** ~40 lines from `advisor-orchestrator.md` /
~1.2k tokens.

## Verification protocol

Each step except 3+4 (MEMORY.md prose-only edits) ends with a fresh-
session `/context` measurement against the committed state. The
sequence is:

1. Apply Step N's changes locally
2. `git add` + commit + push to `governance-v0`
3. Close session
4. Open fresh session
5. `/context` → record Memory files breakdown
6. Compare to predicted savings
7. If ≥ 75% of predicted savings landed → proceed to Step N+1
8. If < 75% → STOP, surface to user, do not proceed

## Hard refusals for next session

- **Never bypass Step 1's sentinel probe.** If `.claude/refs/` auto-
  loads, the entire plan is invalid; cuts must be re-scoped.
- **Never move `decision-queue.md`, `advisor-orchestrator.md`,
  `branch-manager.md`, `multi-lane-worktree.md`, or `pmd-invariants.md`
  out of `.claude/rules/`.** These are read on every polling tick
  / every DQ transition / every BM dispatch. Always-loaded by design.
- **Never trim "Promoted patterns (3+ occurrences)" in MEMORY.md.**
  These are confirmed failure modes by definition.
- **Never trim the active workflow state line** (`ACTIVE: ...`).
- **Never modify auto-mode classifier behavior** to bypass denials.
  If an edit is denied, surface to the user and let them approve
  explicitly (this session: Schema v2 compression denied first
  attempt; landed cleanly after explicit user approval).

## Expected total savings (Option B)

| Step | Source | Predicted savings |
|---|---|---|
| 1 | Sentinel probe (no cuts) | 0 |
| 2 | auto-phase + auto-roadmap move | ~12.5k tokens |
| 3 | MEMORY.md Junior trims | ~250 tokens |
| 4 | MEMORY.md Cargo trims | ~150 tokens |
| 5 | advisor-orchestrator.md §3.7+§3.8 | ~700 tokens |
| 6 | advisor-orchestrator.md §6.1+§6.2 | ~1.2k tokens |
| **Total** | | **~14.8k tokens** |

Brings Memory files from 65k to ~50k (32.6% → ~25% of context).

## What NOT to do in next session (Option C, deferred)

The aggressive option (split `advisor-orchestrator.md` 17k → skeleton +
on-demand refs) is **deferred** until context pressure becomes binding.
Current state: 14.7% free space — uncomfortable but not red. Option C
is a structural reshape with subtle failure modes (forgetting which
ref to load, the skeleton missing a routing branch) — and the harness
is working well, which is the user's framing (2026-05-22).

Re-evaluate Option C only if:
- Sessions regularly hit the autocompact buffer
- The advisor's polling cadence degrades because Read-on-demand of
  procedure refs becomes the bottleneck
- A retro flags "advisor missed routing detail X" ≥ 2× in distinct
  sub-phases

## Companion files / references

- `.claude/rules/advisor-orchestrator.md` §1 — pre-compact handover
  discipline (the rule this handover honors)
- `.claude/lessons/feedback_context_trim_verify_empirically.md` —
  PMD #147, the empirical-verification-before-trim rule
- `.claude/lessons/feedback_principles_not_rules.md` — single-
  occurrence patterns don't belong in always-loaded rules
- `.claude/refs/dq-recipes.md` — precedent for the `.claude/refs/`
  on-demand pattern (already in use)
- MEMORY.md "Historical" section line 186-187 — record of this session's
  Option A archives

## Resume instructions for next session

1. Read this handover file (`.claude/PRPs/handovers/context-prune-option-b-2026-05-22.md`).
2. Confirm Option A landed: `git log --oneline -5 governance-v0` should
   show the MEMORY.md prune + decision-queue.md Schema compression commits.
3. Run `/context` to record the post-Option-A baseline.
4. Execute Steps 1-6 in order, with the verification protocol between
   each step.
5. Update this handover file with actual results vs predictions at
   each step (or author a retro at the end).
