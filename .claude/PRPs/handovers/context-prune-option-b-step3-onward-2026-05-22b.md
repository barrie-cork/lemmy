# Context-prune Option B — handover for next session (Steps 3-6)

> **Authored 2026-05-22 by advisor session on `governance-v0`.**
> Self-contained — readable with zero conversation context.
>
> **Author:** advisor (canonical CWD `brehon-fork`)
> **Branch:** `governance-v0`
> **Last commit before handover:** `17a3eb1bb` (Step 2 ship)

## What this is

A continuation of `.claude/PRPs/handovers/context-prune-option-b-2026-05-22.md`.
Steps 1 (sentinel probe) + 2 (relocate auto-phase + auto-roadmap rules)
are DONE in this session. Steps 3-6 remain. This file is the resume
brief for whichever fresh session picks them up.

## Lanes status (read this FIRST per advisor-orchestrator.md §1 surface-first ritual)

`lanes: brehon-fork:governance-v0 active; other-active:`
- `brehon-fork-fed-in-d` (phase-v1-federation-inbound-d) — merged, lane lingering pending cleanup
- `brehon-fork-quality-r1` (phase-v1-quality-r1) — shipped same day; lane lingering
- `brehon-fork-ship-2` (phase-v1-ship-2) — in flight (per roadmap)
- `brehon-fork-tooling` (tooling-local-validation) — long-lived tooling lane
- `brehon-fork-audit-2026-05-22` (detached HEAD) — audit worktree

This handover work is **canonical-checkout meta-edits only**
(rules, lessons, templates, MEMORY.md). It does NOT touch any phase
branch. The lane sessions (fed-in-d, quality-r1, ship-2) are
independent — do not let session-start ritual on resume conflict
with them.

## Where Steps 1 + 2 landed

| Step | Commit | Result |
|---|---|---|
| 1 — sentinel probe | `f4d584345` (after `a73aa7ddd`) | PASS — `.claude/refs/` does not auto-load |
| 2 — relocate rules | `17a3eb1bb` | `.claude/rules/auto-phase.md` + `.claude/rules/auto-roadmap.md` → `.claude/refs/`; Step 0 Read added to 4 skill bodies; all live cross-refs updated |

### What CHANGED behaviorally (load-bearing for resume)

- `.claude/refs/auto-phase.md` and `.claude/refs/auto-roadmap.md` are
  **no longer auto-loaded at session start**. Confirmed PASS via
  Step 1's sentinel probe.
- Every `/auto-phase`, `/auto-roadmap`, `/roadmap-next` invocation
  now begins with a Phase 0 "Step 0 (MANDATORY)" that Reads the
  matching rule file. The Read is the first action; routing decisions
  follow.
- Skill bodies updated:
  - `~/.claude/commands/auto-phase.md` (user-scope)
  - `~/.claude/commands/roadmap-next.md` (user-scope)
  - `.claude/commands/auto-roadmap.md` (project-scope)
  - `.claude/commands/roadmap-next.md` (project-scope)
- The user-scope edits are at on-disk paths outside the git repo;
  `git status` won't see them but they ARE in place.

### What did NOT change (intentionally preserved)

- Historical artifacts (briefs, retros, reports, lesson files) still
  cite `.claude/rules/auto-*.md` — these are paths-as-of-their-time
  and not load-bearing for current behavior.

## Predicted /context state at resume

The Step 1 baseline at HEAD `f4d584345` was:
- **89k / 200k tokens (45%)** total
- **Memory files: 64.9k tokens (32.5%)**

After Step 2 (HEAD `17a3eb1bb`), expect:
- Memory files: ~52k tokens (~26%) — **save ~12.5k tokens**
- Total: ~76k / 200k (~38%)

**Verify before proceeding to Step 3.** Open a fresh session in
`C:/Users/barri/Developer/brehon-fork`, run `/context`, confirm the
`Memory files` section no longer lists `.claude\rules\auto-phase.md`
or `.claude\rules\auto-roadmap.md`. If they still appear:

- Probably means git push hasn't propagated to your CC's view of the
  repo. Run `git fetch && git pull --ff-only origin governance-v0`
  in the fresh session and re-check.
- If still present after pull, something is wrong — STOP, surface to
  user. Do NOT proceed to Steps 3-6 with stale state.

## Remaining steps (3-6)

### Step 3 — MEMORY.md "Junior daemon" section trims

**Where:** `C:/Users/barri/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/MEMORY.md`

**Edit:** move these 4 entries from the "Junior daemon + workers"
section to the "Historical (>2 weeks)" section at the bottom:

- `feedback_junior_finalize_stage_per_job` — operational texture
- `feedback_junior_finalize_skips_when_worker_pre_pushes` — same
- `feedback_junior_cancel_db_lock_zombie` — recovery recipe, low frequency
- `feedback_junior_task_liveness_check` — diagnostic

**Expected savings:** ~6 lines / ~250 tokens.

**Reasoning preserved:** the entries don't get deleted from PMD; they
just move out of the always-loaded index. Future tasks needing them
will still find via `memory_search_hybrid`.

### Step 4 — MEMORY.md "Cargo / Rust" section trims

Same file. Move these 3 entries to Historical:

- `feedback_e2e_filter_assumes_naming`
- `feedback_rate_limit_debug_config_post_bucket`
- `feedback_pin_strategy_check_derives`

**Why:** these are impl-task reference cards. The impl-task subagent
loads `.claude/lessons/` at task start per the file-class injection
table in `advisor-orchestrator.md` §2.4. The advisor session almost
never compiles, so always-loading them is waste.

**Expected savings:** ~3 lines / ~150 tokens.

### Step 5 — advisor-orchestrator.md §3.7 + §3.8 externalize

**File:** `.claude/rules/advisor-orchestrator.md`

**Two sections to move:**
- §3.7 "Dogfood gate (new slash commands)" — fires only when authoring
  a new `.claude/commands/<verb>.md` (~1× per quarter)
- §3.8 "Schema-changing-spec retrofit gate" — fires only when plan-mode
  produces a new artifact shape (extremely low frequency)

**Move to:** new file `.claude/refs/advisor-narrow-gates.md`.

**Replace the two sections in advisor-orchestrator.md with a single
one-line pointer:**

```markdown
### 3.7 Dogfood gate + 3.8 Schema-retrofit gate

Both gates fire only when authoring new slash commands or when plan-mode
produces a new artifact shape. Procedure: `.claude/refs/advisor-narrow-gates.md`.
```

**Expected savings:** ~25 lines / ~700 tokens.

**Hazard:** the §3.7 + §3.8 procedures cite specific feedback memory
files. Make sure the externalized refs/advisor-narrow-gates.md is
self-contained — Read both sections in the current rule file, copy
them verbatim into the new ref file, then replace.

### Step 6 — advisor-orchestrator.md §6 "Subagent delegation" externalize

**File:** `.claude/rules/advisor-orchestrator.md`

**Three sub-sections:**
- §6.1 "Parallel dispatch" status: `defer-pending-2nd-recurrence`
- §6.2 "Verify-after-subagent-completes" status: `record-only, single occurrence`
- §6.3 "Bounded sub-agent dispatch and report semantics" — always-applicable

Per `feedback_principles_not_rules.md`: single-occurrence patterns
don't belong in always-loaded rules.

**Move §6.1 + §6.2 to:** new file `.claude/refs/advisor-subagent-dispatch.md`.

**Keep §6.3 inline** in advisor-orchestrator.md — it's the always-applicable
subset (brief sub-agents like smart colleague; trust but verify).

**Replace §6.1 + §6.2 with one-line pointer.**

**Expected savings:** ~40 lines / ~1.2k tokens.

When the second recurrence of §6.1 or §6.2 pattern occurs (which
will happen in some future session), lift back from refs/ into the rule.

## Verification protocol per step

After each step (3-6), commit + push to `governance-v0`, then verify
in a fresh session via `/context`:

1. Apply Step N's changes locally.
2. `git add` + commit + push.
3. Close session.
4. Open fresh session in `C:/Users/barri/Developer/brehon-fork`.
5. Run `/context`. Record `Memory files` total.
6. Compare to predicted savings.
7. **If ≥75% of predicted savings landed → proceed to Step N+1.**
8. **If <75% → STOP, surface to user, do not proceed.**

Steps 3 + 4 are MEMORY.md prose-only edits and probably won't need
a fresh-session verification round if the user is OK with the prose-
edit being self-verified.

## Hard refusals for next session

- **Never trim "Promoted patterns (3+ occurrences)" in MEMORY.md.**
  These are confirmed failure modes by definition.
- **Never trim the "ACTIVE: ..." workflow state line.**
- **Never move other rule files** (`.claude/rules/advisor-orchestrator.md`,
  `branch-manager.md`, `decision-queue.md`, `multi-lane-worktree.md`,
  `pmd-invariants.md`) — they're read on every polling tick and are
  always-loaded by design. The handover originally said this; it's
  reaffirmed here.
- **Never bypass empirical verification.** Step 1 (sentinel probe)
  is the validation gate for the entire plan. Don't trust documentation
  claims about what loads — verify in a fresh session before relying
  on the claim.
- **Never modify auto-mode classifier behavior** to bypass denials.

## Expected total savings (Steps 3-6)

| Step | Source | Predicted savings |
|---|---|---|
| 3 | MEMORY.md Junior trims | ~250 tokens |
| 4 | MEMORY.md Cargo trims | ~150 tokens |
| 5 | advisor-orchestrator.md §3.7+§3.8 | ~700 tokens |
| 6 | advisor-orchestrator.md §6.1+§6.2 | ~1.2k tokens |
| **Subtotal Steps 3-6** | | **~2.3k tokens** |
| **Step 2 already shipped** | | **~12.5k tokens** |
| **Option B total (Steps 2-6)** | | **~14.8k tokens** |

## Decisions taken this session that may affect future sessions

### Step 2's mid-phase-edit semantics change

The user-scope `~/.claude/commands/auto-phase.md` §6 "Mid-phase skill-edit
deferred-effect awareness" block was updated to reflect that:
- Skill body + JSON template = sticky (loaded once at session start)
- Rule file (now at .claude/refs/auto-phase.md) = re-read per-invocation

This is a **faster turnaround for rule edits** than the pre-move
behavior. It means a concurrent meta-editor session can author a rule
edit and have it take effect on the next /auto-phase tick (typically
within minutes) rather than waiting for the in-flight advisor to
restart. The trade-off is that a mid-flight rule edit may surface
mid-phase in non-obvious ways.

The block is documented; future sessions should expect this behavior.

### Tasks 1 + 2 completed in this session

Task list captured by the harness — visible in the next session via
TaskList if the harness preserves them. If not, re-create from this
handover:

1. Step 1 — Sentinel-probe `.claude/refs/` (DONE, PASS at `f4d584345`)
2. Step 2 — Move auto-phase + auto-roadmap to refs/ (DONE at `17a3eb1bb`)
3. Step 3 — MEMORY.md Junior daemon trims (PENDING)
4. Step 4 — MEMORY.md Cargo/Rust trims (PENDING)
5. Step 5 — advisor-orchestrator §3.7+§3.8 externalize (PENDING)
6. Step 6 — advisor-orchestrator §6.1+§6.2 externalize (PENDING)

## Cross-references

- `.claude/PRPs/handovers/context-prune-option-b-2026-05-22.md` —
  original handover authored by prior session (Option A done; this is
  Option B Steps 1-6, of which 1-2 are now done).
- `.claude/rules/advisor-orchestrator.md` §1 "Pre-compact handover
  discipline" — the rule mandating self-contained handovers.
- `.claude/rules/handover.md` — handover shape invariants.
- `.claude/lessons/feedback_context_trim_verify_empirically.md` (PMD #147)
  — the empirical-verification-before-trim rule.
- `.claude/lessons/feedback_principles_not_rules.md` — the doctrine
  Step 6 leans on.
- `.claude/refs/auto-phase.md` (relocated) — read by `/auto-phase` Phase 0 Step 0.
- `.claude/refs/auto-roadmap.md` (relocated) — read by `/auto-roadmap`,
  `/roadmap-next` Phase 0 Step 0.

## Resume instructions for next session

1. **Read this file first** (`.claude/PRPs/handovers/context-prune-option-b-step3-onward-2026-05-22b.md`).
2. Read the original handover for Option B context:
   `.claude/PRPs/handovers/context-prune-option-b-2026-05-22.md`.
3. Confirm Step 2 landed:
   ```bash
   git log --oneline -3 governance-v0 | head -5
   ```
   Expect `17a3eb1bb chore(refs): relocate auto-phase + auto-roadmap`
   somewhere in the recent history.
4. Run `/context`. Confirm `.claude\rules\auto-phase.md` and
   `.claude\rules\auto-roadmap.md` are NOT in the Memory files list.
   If still listed, STOP and surface to user.
5. Execute Step 3 (MEMORY.md Junior trims) — it's a pure prose edit
   to MEMORY.md, no fresh-session verification required.
6. Execute Step 4 (MEMORY.md Cargo trims) — same.
7. For Step 5 + 6: each ends with a fresh-session `/context` measurement.
8. After all steps land, author a retro at
   `.claude/PRPs/reports/session-retro-2026-05-22-context-prune-option-b-steps3to6.md`
   capturing actual savings vs prediction.

Total wall-clock estimate for Steps 3-6 in fresh session: ~30-45 minutes
if no surprises (most of that is fresh-session `/context` polls).
