# Spec: `/roadmap-next` + `/auto-roadmap` skill pair

> **Status:** DRAFT (feasibility-assessment). Not yet implemented.
> **Authored:** 2026-05-22
> **Decided by:** user AskUserQuestion answers 2026-05-22
> **Scope:** RT lane (v1-RT-r2 → r5) first; SR lane (v1-ship-2/3) deferred to user decision after assessment

---

## 1. Goal

Adapt `/auto-phase` (single-sub-phase orchestrator) to a roadmap-driven two-skill chain that drives multiple sub-phases of a v1 PRD lane to completion under the four-role model, with user gates only for the six mandatory decisions plus a roadmap-level "approve next sub-phase" gate.

Per user 2026-05-22 AskUserQuestion answers:

- **Scope:** multi-sub-phase chain (single roadmap-driven invocation per sub-phase, not per-lane)
- **Target:** RT lane (r2/r3/r4/r5)
- **Plan-gap handling:** auto-dispatch planning Junior if `<phase>.plan.md` missing
- **Shape:** new skills on top of `/auto-phase`; no edits to existing skill body
- **Worktree model:** two skills, hand-off via user opening Claude Code in the cut worktree

---

## 2. The two skills

### 2.1 Skill 1: `/roadmap-next` (canonical-checkout)

Runs in the canonical `C:/Users/barri/Developer/brehon-fork` checkout on `governance-v0`.

**Inputs:** none (reads roadmap + git state).

**Job:**

1. Read `.claude/PRPs/v1-roadmap.json`.
2. Compute the next eligible sub-phase per the roadmap's `what_remains.high_priority_unstarted` + `implementation_steering.next_logical_sub_phase` fields, modulo any `skipped` lanes (federation-inbound, quality currently).
3. Surface recommendation via `AskUserQuestion` with three options:
   - **Confirm** the recommendation
   - **Pick a different sub-phase** (free-text via "Other")
   - **Defer** (exit without cutting)
4. On confirm:
   - Dispatch `branch-manager` subagent to run `bm-cut <sub-phase>` (existing skill — borrows BM patterns per user steer 2026-05-22). The subagent cuts `phase-<sub-phase>` off `governance-v0` locally + pushes to origin.
   - Run `git worktree add ../brehon-fork-<lane-suffix> phase-<sub-phase>` from canonical.
   - Walk the lane-bootstrap checklist at `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` steps 5-10: copy `.mcp.json.example` → `.mcp.json`, wire `pmd-canonical-guard.sh` + `session-start-multi-lane-check.sh` SessionStart hooks into `.claude/settings.local.json`, run the programmatic verification probes from steps 8 + 10.
   - Update `v1-roadmap.json`: flip the sub-phase's `status` from `unstarted` → `in_flight` and add the worktree path under a new field `worktree`.
   - Commit + push the roadmap update to `governance-v0` (canonical-checkout meta-edit, allowed per `multi-lane-worktree.md` §"Lifecycle" + Hard refusal #2 carve-out for `governance-v0` meta-work).
5. Print a one-screen handoff:

   ```
   === Lane cut: phase-v1-RT-r2 ===
   Worktree:  C:/Users/barri/Developer/brehon-fork-rt-r2
   Branch:    phase-v1-RT-r2 (pushed to origin)
   Bootstrap: ✓ .mcp.json   ✓ pmd-canonical-guard wired   ✓ multi-lane-check wired

   Next steps (manual):
     1. Open Claude Code in C:/Users/barri/Developer/brehon-fork-rt-r2
     2. Run /auto-roadmap (skill 2 — reads its sub-phase from the lane's branch name)
   ```

**Refusals:**

- Roadmap file missing or unparseable → STOP.
- `governance-v0` working tree dirty → STOP (existing `bm-cut` refusal carries through).
- Recommended sub-phase has no PRD section in roadmap → STOP.
- A worktree already exists at `../brehon-fork-<lane-suffix>` → STOP, surface the existing path.

**Reuse:** This skill is ~70% existing BM patterns. `bm-cut` is dispatched verbatim. The only NEW work is roadmap-read + AskUserQuestion + worktree-add + bootstrap-checklist walk + roadmap update.

---

### 2.2 Skill 2: `/auto-roadmap` (lane worktree)

Runs in a lane-dedicated worktree (e.g. `C:/Users/barri/Developer/brehon-fork-rt-r2`) on its `phase-v1-<sub-phase>` branch.

**Inputs:** none (derives sub-phase from `git branch --show-current`).

**Job:**

1. **Phase 0 — Identify sub-phase.** `git branch --show-current` → `phase-v1-<sub-phase>`. Extract `<sub-phase>` (e.g. `v1-RT-r2`). Read its entry from `.claude/PRPs/v1-roadmap.json` (the canonical-checkout file is visible through the worktree's `.git/`). Verify `status == "in_flight"`. If `done` or `unstarted` → STOP (skill 1 should have flipped it to `in_flight`).

2. **Phase 0.5 — Plan-gap check.** Glob `.claude/PRPs/plans/<sub-phase>*.plan.md`.

   - **Plan exists** → proceed to Phase 1.
   - **Plan missing** → auto-dispatch planning Junior per user decision:
     - Read the PRD section for this sub-phase (e.g. `.claude/PRPs/prds/v1-reputation-tuning.prd.md` §"v1-RT-r2 scope" — assumed structure).
     - Author a planning brief at `.claude/PRPs/briefs/<sub-phase>-planning-1.md` using the template at `.claude/PRPs/templates/planning-brief.template.md` (TBD — may need to be authored as part of this spec).
     - **Run `/brehon-clarify` on the brief** per advisor-orchestrator.md §3.3 (clarify gate is mandatory before planning).
     - Resolve every clarify-DQ (advisor-mode self-answer with PRD citation, or user-relay via AskUserQuestion if mode=user-relay).
     - Dispatch `[role:planning]` Junior task.
     - Wait for done (ScheduleWakeup 1200s initial, 600s subsequent).
     - On done, run DoD smoke + watchpoint specificity gate from advisor-orchestrator.md §3.4 + §3.5.
     - **User gate (Plan approval)** — AskUserQuestion surface plan + DoD result.
     - On approval, proceed to Phase 1.

3. **Phase 1 — Delegate to `/auto-phase`.** Invoke `/auto-phase <sub-phase>` via Skill tool. `/auto-phase` runs unchanged: bm-cut is skipped (`--no-bm-cut` flag — phase branch already exists from skill 1), planning is already done by step 2, so the skill enters at `impl-cohort-1` state.

   Critical detail: skill 2 dispatches `/auto-phase --no-bm-cut --start-from impl-cohort-1` to ensure the auto-state JSON is initialized at the right stage. (Or, equivalently, write a pre-seeded auto-state file before invocation — see §3.4 below.)

   The five remaining user gates (e2e local-vs-dispatch, CR triage, merge confirm, retro sign-off, and the implicit ADR-affecting DQ gate) fire normally inside `/auto-phase`. Skill 2 is the parent — it sees `/auto-phase` return when the sub-phase reaches `done`.

4. **Phase 2 — Mark sub-phase done in roadmap.** When `/auto-phase` returns with state `done`:
   - Update `.claude/PRPs/v1-roadmap.json`: flip the sub-phase's `status` from `in_flight` → `done`; populate `pr`, `merge_commit`, `retro`, `verify` from the auto-state JSON.
   - Commit + push to `governance-v0` from the worktree (the roadmap file lives at the worktree's `.claude/PRPs/v1-roadmap.json`, which IS the canonical file via shared `.git/`).
   - This update is a `chore(advisor)` commit, attributable per `decision-queue.md` Attribution integrity rules.

5. **Phase 3 — Surface next sub-phase.** Print:

   ```
   === /auto-roadmap v1-RT-r2 — done ===
   Merged: PR #<N>  ·  Retro: <path>  ·  Verify: <path>

   Next unstarted sub-phase in roadmap: v1-RT-r3
   To continue:
     1. Close Claude Code in this worktree
     2. Open Claude Code in C:/Users/barri/Developer/brehon-fork (canonical)
     3. Run /roadmap-next
   ```

   `/auto-roadmap` does NOT loop into the next sub-phase. It's strict single-sub-phase per worktree. Looping would require keeping the worktree open across sub-phases, which conflicts with `multi-lane-worktree.md` §"Lifecycle" step 3 (remove worktree after merge).

**Refusals:**

- Branch is `governance-v0` or `main` → STOP (`/auto-roadmap` is for lane worktrees only; user should run `/roadmap-next` from canonical).
- `phase-<sub-phase>` branch matches but roadmap entry `status != "in_flight"` → STOP.
- Plan missing AND PRD missing → STOP, surface "PRD must exist before /auto-roadmap can dispatch planning".

---

## 3. Feasibility assessment

### 3.1 Goal alignment (vs `/auto-phase` design intent)

| `/auto-phase` design goal | `/roadmap-next` + `/auto-roadmap` alignment |
|---|---|
| Single-trigger sub-phase orchestrator (one invocation = one bm-cut→merge→retro) | ✓ `/auto-roadmap` is exactly this, with skill 1 splitting bm-cut + worktree-add upstream |
| Six mandatory user gates preserved | ✓ All six fire inside `/auto-phase`; skill pair adds one upstream gate (roadmap recommendation confirmation) |
| Advisor never authors content | ⚠️ Plan-gap handling auto-dispatches planning Junior — planner Junior authors plan; advisor authors only the planning brief. Brief-authoring is already advisor work (per advisor-orchestrator.md §2). No new four-role violations. |
| Subagent delegation respects four-role model | ✓ `bm-cut` Junior (BM role), `planning` Junior (planner role), `/auto-phase` reuses impl-task + ci-watcher + bm-task Juniors. No new roles introduced. |
| Skill body changes deferred-effect | ✓ No edits to `/auto-phase` skill body. Skill 2 invokes `/auto-phase` as a black box. |
| Token-frugal on resume | ✓ Skill 2's `/auto-phase` invocation inherits all Phase 0.5 + Phase 0.7 resume disciplines unchanged. |
| Cadence respects prompt-cache TTL | ✓ All cadence rules in `/auto-phase` Phase 3 apply unchanged. |

**Feasibility verdict: HIGH.** The two-skill chain composes cleanly with `/auto-phase`'s existing single-sub-phase contract. The only new mechanic is the plan-gap auto-dispatch in skill 2 Phase 0.5, which is itself an extension of patterns already in advisor-orchestrator.md.

### 3.2 Risk assessment

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| **Roadmap-update race** with another session also editing the roadmap file | LOW | MEDIUM | Per `multi-lane-worktree.md` Hard refusal #6 atomic protocol: read-mutate-commit in single shell sequence; `git status` between add and commit. Mitigated by the fact that roadmap edits are rare (once per sub-phase boundary). |
| **Plan-gap auto-dispatch misreads PRD section structure** | MEDIUM | MEDIUM | The PRD for RT lane (`v1-reputation-tuning.prd.md`) may not have per-sub-phase scope sections in a uniform format. Skill 2 Phase 0.5 must read the PRD § and extract scope; if the PRD's structure varies, the brief will be malformed. **Mitigation**: skill 2 surfaces the auto-authored brief via AskUserQuestion BEFORE dispatching planning Junior; user can edit or abort. Adds 1 user gate per missing-plan case. |
| **`/auto-phase --no-bm-cut --start-from impl-cohort-1` flag combo** untested | MEDIUM | LOW | The flag combo is supported per `/auto-phase` skill body lines 8-9, but the rule (`auto-phase.md` hard refusal #7) says `--no-bm-cut` requires a matching `phase-<phase>` branch — skill 1 ensures this. Verify in dogfood. |
| **Roadmap drift if user mutates roadmap outside skill** | MEDIUM | LOW | Roadmap is a tracked git file; concurrent mutations show in git log. Skill 1 + 2 do `git fetch + read` immediately before mutating. |
| **PRD for SR lane has 3 unstarted sub-phases but no skeleton plans** | KNOWN | (deferred per user) | User picked RT for first target; SR deferred. RT lane's per-sub-phase scope is encoded in the entry-kind registry (8 pre-landed consts each named for r2/r3/r4/r5). Skill 2 can derive scope from the registry for RT sub-phases. |
| **`/auto-phase` skill-body mid-phase edit invariant** (lines 68-75): in-flight `/auto-phase` session cannot pick up skill edits until restart | LOW | LOW | Skill 1 + 2 don't edit `/auto-phase`. Invariant is preserved. |
| **Worktree pruning at sub-phase end** | LOW | LOW | Per `multi-lane-worktree.md` §"Lifecycle" step 3, user removes worktree after merge. Skill 2 surfaces the next sub-phase but does NOT auto-remove its own worktree (write-while-in-use risk). User manually runs `git worktree remove ../brehon-fork-<lane>` from canonical before running `/roadmap-next` again. |
| **Roadmap-level user gate fatigue** | LOW | LOW | One gate per sub-phase boundary (skill 1's "confirm recommendation") adds to the six per-sub-phase gates. Total: ~7 gates per sub-phase, ~28 gates across RT-r2 → r5. Acceptable. |

### 3.3 Implementation effort estimate

| Component | Effort | Notes |
|---|---|---|
| Skill 1 (`/roadmap-next.md`) skill body | ~2-3 h | Borrows `bm-cut.md` shape (~150 lines). Adds roadmap-read, AskUserQuestion, worktree-add wrapper. |
| Skill 2 (`/auto-roadmap.md`) skill body | ~4-6 h | More complex — Phase 0.5 plan-gap handler is new logic. Phase 1 is a thin wrapper over `/auto-phase`. |
| Planning brief template for auto-authored briefs | ~1-2 h | Probably already exists at `.claude/PRPs/templates/` — check before authoring. |
| Roadmap-update commit attribution + Hard refusal protocol | ~30 min | Mechanical; encoded in `multi-lane-worktree.md` Hard refusal #6 + `decision-queue.md` Attribution integrity. |
| Dogfood + retro | ~1 day | Run on v1-RT-r2 end-to-end; retro the pair (per `feedback_dogfood_slash_command_specs.md`). |
| **Total** | **~2 days of advisor wall-clock** | Mostly composition + dogfood. |

### 3.4 Critical implementation detail: how skill 2 invokes `/auto-phase`

Three options, in order of preference:

1. **Pre-seed auto-state JSON before invoking `/auto-phase`** (RECOMMENDED).
   Skill 2 Phase 0.5 (after planning is done + plan approved) writes `.claude/auto-state/<sub-phase>.json` directly with `stage: "impl-cohort-1"`, then invokes `/auto-phase <sub-phase>`. `/auto-phase` detects the state file exists, routes through Phase 0.5 reconciliation, sees stage = `impl-cohort-1`, and proceeds.
   **Pro:** uses `/auto-phase`'s existing resume path; no flag-combo surprises.
   **Con:** depends on auto-state JSON schema stability.

2. **Use `--no-bm-cut --start-from impl-cohort-1` flags.**
   `/auto-phase --no-bm-cut --start-from impl-cohort-1 v1-RT-r2`.
   **Pro:** explicit; no JSON-mutation by skill 2.
   **Con:** flag-combo per `/auto-phase` skill body lines 8-9 is supported but per the rule (`auto-phase.md` hard refusal #6), `--start-from` is intended for catch-fire recovery, not fresh-start at a non-init stage. Semantic mismatch.

3. **Invoke `/auto-phase` with no flags; it does its own bm-cut, which becomes a no-op because the branch exists.**
   **Pro:** simplest invocation.
   **Con:** `bm-cut` will refuse if branch already exists (per `phase-branch.md` discipline). Won't work without skill body changes.

**Recommendation:** option 1. Skill 2 writes the auto-state JSON; `/auto-phase` resumes into the impl-cohort-1 state.

### 3.5 Alternate target lanes

User picked RT for first target. Other lanes per the roadmap:

- **SR (ship-readiness)**: `v1-ship-2` + `v1-ship-3` unstarted. PRD scope per `v1-ship-readiness.prd.md` §1.1 names 3 sequenced sub-phases; ship-1 (+ r1/r2 follow-ups) shipped. Scope for ship-2/3 must be derived from PRD §2/§3 — recommend reading PRD before extending skill 2's PRD-scope-extraction code to support SR.
- **RT (reputation-tuning)**: r2 (decay calc go-live) → r3 (cron + post-decision hook) → r4 (sponsor allowlist) → r5 (rollup cron). Scope already encoded in `.claude/rules/governance-log-entry-kind-registry.md` §"v1-RT-r1 entry kinds" — each pre-landed const names its target sub-phase + handler file. **Skill 2 can derive RT sub-phase scope mechanically from the registry**; this is a significant simplification vs SR.

This makes RT the right first target. SR can be added later once the skill pair is dogfooded on RT.

---

## 4. Files to create / modify

### Create

- `~/.claude/commands/roadmap-next.md` — skill 1 body (user-scope)
- `~/.claude/commands/auto-roadmap.md` — skill 2 body (user-scope)
- (possibly) `.claude/PRPs/templates/planning-brief-from-roadmap.template.md` — if no existing template fits
- `.claude/rules/auto-roadmap.md` — companion rule with hard refusals + state-routing invariants (mirror `auto-phase.md` rule pattern)

### Modify

- `.claude/PRPs/v1-roadmap.json` — add `worktree` field to each sub-phase entry shape; bump `$schema_version` to 2
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` — extend to note this checklist is now also invoked from `/roadmap-next`

### NOT modify

- `~/.claude/commands/auto-phase.md` — preserved verbatim per user decision
- `.claude/rules/auto-phase.md` — preserved verbatim per user decision
- `.claude/PRPs/templates/auto-phase-state.template.json` — preserved verbatim
- Any of the BM verb scripts at `.claude/commands/bm/*.md` — `bm-cut` reused unchanged

---

## 5. Dogfood plan

Per `feedback_dogfood_slash_command_specs.md` — every new skill must include a pre-commit dogfood walking the skill against a real input.

**Skill 1 dogfood target:** the existing roadmap file. Walk: read roadmap → recommend next sub-phase (expect: `v1-RT-r2`) → user confirms → `bm-cut` dispatch → worktree-add → bootstrap-walk → roadmap update.

**Skill 2 dogfood target:** the cut RT-r2 worktree. Walk: identify sub-phase from branch → plan-gap check (expect: missing) → read PRD § for RT-r2 (registry-derived) → author planning brief → clarify gate → planning Junior dispatch → user gate → `/auto-phase` invocation via pre-seeded auto-state JSON → wait for `done` → roadmap update.

**Dogfood report goes into:** the skill body's `## Pre-commit dogfood` section (mirroring `/auto-phase` lines 571-590).

---

## 6. Decision points before implementation

These should be resolved before authoring the skill bodies:

1. **Roadmap schema extension.** The `worktree` field — should it be on the sub-phase entry, or in a sibling top-level map keyed by sub-phase? Recommendation: on the sub-phase entry, alongside `pr`, `merge_commit`, `retro`, `verify`. Bump `$schema_version: 1` → `2`.

2. **Planning brief template existence.** Check `.claude/PRPs/templates/` for an existing planning-brief template. If none fits the auto-authored-from-PRD case, a new one is needed.

3. **PRD scope extraction for RT.** Skill 2 derives RT sub-phase scope from `governance-log-entry-kind-registry.md` rows. For RT-r2 specifically, the const `_DECAY_KNOB_CHANGED` names `v1-RT-r2 admin_config.rs` as the fire site. Is the full PRD scope for RT-r2 captured there, or only the const fire site? **Read `v1-reputation-tuning.prd.md` §"RT-r2" before authoring skill 2's PRD-scope extractor.**

4. **`/auto-phase` invocation mechanism.** Option 1 (pre-seed auto-state) vs option 2 (`--start-from` flag) vs option 3 (`bm-cut` no-op). Recommend option 1.

5. **Hard refusal on `/auto-phase` skill version drift.** Skill 2 invokes `/auto-phase` indirectly via Skill tool. If `/auto-phase` skill body changes between dogfood and ship, skill 2 may break. Consider a `/auto-phase` version pin in skill 2's spec (e.g. "verified against /auto-phase as of trunk SHA `<X>`").

---

## 7. Out of scope

Per user 2026-05-22 steers + the four-role model:

- **Auto-merging across sub-phases.** Each sub-phase still hits the merge-confirm user gate.
- **Cross-lane orchestration.** Each invocation of skill 1 / 2 handles one sub-phase, one lane.
- **Auto-removing worktrees** after sub-phase merge. User runs `git worktree remove` manually.
- **Auto-running `/brehon-phase-transition`.** That skill is its own thing; `/auto-phase` already calls it in its terminal stage.
- **PRD authorship.** Skill 2 reads PRDs to extract scope; never writes them.
- **`/auto-phase` skill body edits.** Preserved verbatim.

---

## 8. Open questions

1. What does `v1-reputation-tuning.prd.md` actually say about RT-r2 through r5? The roadmap pulls scope from the entry-kind registry, but the PRD may have additional context (DTOs, migration shape, feature flag wiring) that the registry doesn't capture. **Read the PRD before authoring skill 2.**

2. Should skill 1 also handle the SR lane's missing PRD sub-phase scope (ship-2 + ship-3)? Or only RT initially? **User decision deferred; first-target is RT per AskUserQuestion answer.**

3. Should the skill pair be authored as user-scope (`~/.claude/commands/`) or project-scope (`.claude/commands/`)? `/auto-phase` is user-scope; recommend same shape.

4. Is the lane-suffix derivation deterministic? `v1-RT-r2` → `rt-r2`? `v1-ship-2` → `ship-2`? Codify in skill 1 body.

---

## See also

- `~/.claude/commands/auto-phase.md` — the skill this pair composes on top of
- `.claude/rules/auto-phase.md` — the orchestration state-machine rule
- `.claude/rules/multi-lane-worktree.md` — worktree discipline skill 1 inherits
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` — the bootstrap checklist skill 1 walks
- `.claude/commands/bm/bm-cut.md` — the BM pattern skill 1 borrows from
- `.claude/PRPs/v1-roadmap.json` — the file skill 1 reads and skill 2 updates
