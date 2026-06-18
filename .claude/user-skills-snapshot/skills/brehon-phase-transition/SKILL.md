---
name: brehon-phase-transition
description: "Automate the Brehon governance platform phase or sub-phase transition handoff, entirely within the brehon-fork advisor CWD. Produces the four deliverables that close one phase/sub-phase and bootstrap the next advisor session: close the completing workflow-state record, create a fresh workflow-state skeleton, write the merged in-repo bootstrap file (.claude/PRPs/handovers/<id>-bootstrap.md), and update the brehon-fork MEMORY.md index. Use when: 'phase transition', 'sub-phase transition', 'transition to phase N', 'transition to v1-JM-d', 'handoff to phase N', 'handoff to v1-JM-d', 'close phase N and open phase M', 'close v1-JM-c and open v1-JM-d', 'brehon phase transition', or any request to prepare the next Brehon phase or sub-phase advisor session. Also use when the user says a phase or sub-phase is complete and wants to set up the next one."
---

# Brehon Phase Transition

You are the **system orchestrator** performing a phase or sub-phase transition for the Brehon governance platform — a Lemmy 1.0-beta fork at `barrie-cork/lemmy` on branch `governance-v0`. The advisor session runs **inside the brehon-fork checkout** (CWD `C:/Users/barri/Developer/brehon-fork`, or a lane-dedicated worktree per `.claude/rules/multi-lane-worktree.md`). This skill produces the handoff artifacts — all of them in or under that CWD — so one advisor session closes cleanly and a fresh session picks up the next phase with full context.

> **Migration note (2026-05-19).** This skill was rewritten to be **brehon-fork-native**. The historical homeserver advisor-context chain (`homeserver/.claude/advisor-context-*.md`, `homeserver/.claude/archive/`) and the homeserver-PMD `project_brehon_*` notes/complete files are **frozen historical records** — this skill no longer reads or writes them. The brehon-fork checkout is the sole source of truth. The last homeserver-side artifacts (the dormant SL-e archive + the v1-federation-inbound-b advisor-context written at the fed-in-a→fed-in-b transition) stay where they are; nothing migrates them. From the fed-in-b→fed-in-c transition onward, everything is brehon-fork-side.

## Usage

```
/brehon-phase-transition <completing-id> <next-id>
```

Examples:
- v0 phase boundaries: `/brehon-phase-transition 2b 3`, `/brehon-phase-transition 6 7`
- v1 sub-phase boundaries: `/brehon-phase-transition v1-JM-c v1-JM-d`, `/brehon-phase-transition v1-federation-inbound-b v1-federation-inbound-c`
- mixed: `/brehon-phase-transition 7 v1-AD-a` (when v0 closes and v1 begins)

If the user doesn't provide phase identifiers, ask for them before proceeding.

**Pre-flight CWD check.** Run `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current`. The transition must run from a session whose CWD is the canonical `brehon-fork` checkout on `governance-v0` (the meta-edit lane per `multi-lane-worktree.md`) — that is where the bootstrap file + MEMORY.md commit lands. If the completing phase ran on a lane worktree, that lane's worktree is removed by the user *after* this transition's commit (it is not this skill's job to remove it; surface the `git worktree remove` line as a closing note).

## Phase-id grammar (load-bearing)

A phase-id is one of:
- v0-style: a bare number with optional letter suffix — `5`, `6`, `5a`, `7a`
- v1-style sub-phase: `v1-<lane>-<slice>` — `v1-JM-a`, `v1-JM-c`, `v1-AD-a`, `v1-SL-a`, `v1-federation-inbound-b`, `v1-rep-tuning-r3` (the lane portion may be multi-segment)

Two derived forms are used as filename fragments:

- **`<phase-id>` (display form):** the id as written above. Used in the bootstrap filename (`.claude/PRPs/handovers/<phase-id>-bootstrap.md`) and in human-facing prose.
- **`<phase-slug>` (memory form):** the id with hyphens replaced by underscores. Used in the brehon-fork PMD workflow-state filename (`workflow_state_<phase-slug>.md`). Hyphens are not legal in PMD memory filenames; underscore is the canonical separator.

Example transformations:
- `6` → display `6`, slug `6` (unchanged)
- `5a` → display `5a`, slug `5a` (unchanged)
- `v1-JM-c` → display `v1-JM-c`, slug `v1_JM_c`
- `v1-federation-inbound-b` → display `v1-federation-inbound-b`, slug `v1_federation_inbound_b`

The bootstrap file always uses the display form in its filename (it's a tracked in-repo artifact under `.claude/PRPs/handovers/`, where hyphens are fine). The workflow-state PMD memory file always uses the slug form. No `phase_`-prefix legacy split is needed — that was a homeserver-PMD artifact and does not apply to the brehon-fork PMD or the in-repo handover files. (Historical v0 `project_brehon_phase_<N>_*.md` files live frozen in the homeserver PMD and are out of this skill's scope per the Migration note.)

## File locations

All paths are **brehon-fork-relative** (`C:/Users/barri/Developer/brehon-fork` is the repo root) **except** the workflow-state PMD memory files, which live in the brehon-fork PMD memory directory. Substitute `<phase-id>` and `<phase-slug>` per the grammar above.

- **Bootstrap / handoff file (THE next-session entry point — tracked):**
  `.claude/PRPs/handovers/<phase-id>-bootstrap.md`. This single file merges what
  used to be two separate homeserver artifacts (the advisor-context texture + the
  pasted bootstrap prompt). It is version-controlled, survives session loss, and
  is read at the next session's start. Its own prior instance
  (`<completing-id>-bootstrap.md`) **stays in place** as the historical handoff —
  it is its own archive (git history is the archive); no move/rename.
- **Workflow-state record (running facts — brehon-fork PMD memory):**
  `C:\Users\barri\.claude\projects\C--Users-barri-Developer-brehon-fork\memory\workflow_state_<phase-slug>.md`.
  This is the brehon-fork analogue of the old homeserver `project_brehon_<slug>_notes.md`.
  Auto-loads via the brehon-fork MEMORY.md "Active workflow state" section.
- **Brehon-fork MEMORY.md index:**
  `C:\Users\barri\.claude\projects\C--Users-barri-Developer-brehon-fork\memory\MEMORY.md`
  — the "Active workflow state" section is the only part this skill edits.
- **Retro:** `.claude/PRPs/reports/<phase-id>-retro.md` (the canonical location for
  v1 sub-phases; some early v0 phases live at `.claude/PRPs/retros/phase-<id>-retro.md`
  or `.claude/PRPs/reports/phase-<id>-retro.md` — check `reports/` then `retros/`).
- **Sub-phase plan:** `.claude/PRPs/plans/<phase-id>.plan.md` (or it doesn't exist
  yet — note that the next session must run `/prp-core:prp-plan <next-id>` before
  queueing impl).
- **Completion report (if any):** `.claude/PRPs/reports/<phase-id>-*-report.md`.
- **Implementation plan (vendored in fork):**
  `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` (§3 phase-by-phase).
  This is the brehon-fork vendored copy — the canonical homeserver design-doc suite
  is no longer consulted by this skill.
- **Lessons corpus (the durable cross-phase knowledge layer):**
  `.claude/lessons/feedback_*.md` + `.claude/lessons/reference_*.md`. The bootstrap
  file **references these by filename**, never duplicates them. The brehon-fork
  PMD (`memory_search_hybrid`) indexes them.

## Branching and PR policy (Phase 5 onwards + v1 sub-phases)

Each phase runs on its own branch and closes via PR into `governance-v0`. `.coderabbit.yaml` auto-reviews PRs into `governance-v0`; direct commits to `governance-v0` silently skip review.

- **Phase branch:** `phase-<phase-id>` for v0 phases. For v1 sub-phases the brehon-fork convention is `phase-v1-<lane>-<slice>` (e.g. `phase-v1-federation-inbound-b`). Branch from `governance-v0` at bm-cut time.
- **Impl runs on the phase branch** (or a lane worktree on it). All impl commits land there.
- **Phase close:** the BM subagent opens the PR `phase-<id>` → `governance-v0` via `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head <phase-branch>`. CodeRabbit auto-reviews.
- **Merge after review.** Advisor reviews CR findings, addresses material ones with fix-in-PR commits, merges (user-gated).
- **Grandfathered phases:** Phases 1–4 shipped direct to `governance-v0`; not retroactively PR'd.

## The four deliverables

Execute these in order. Each depends on artifacts read or produced by the previous step. (The old skill had five steps; the homeserver advisor-context archive step is gone — in brehon-fork the prior bootstrap file IS its own archive — and the advisor-context + bootstrap-prompt steps merge into one tracked file.)

### Step 0 — Gather context

**Retro gate (first check, before any reads):** verify the retro file exists and contains the three required reflections — surprised / change / carry-forward — per `feedback_retro_not_report.md`. They may appear as literal H2 headers (`## What surprised us`, `## What to change`, `## What to carry forward`) OR as an equivalent labelled structure (some four-role retros use an 8-section shape where these map to "What surprised us / What to change / What to carry forward" subsections — accept that as satisfying the gate provided all three reflections are present and substantive).

Retro filename resolution (check in this order):
1. `.claude/PRPs/reports/<completing-id>-retro.md`
2. `.claude/PRPs/reports/phase-<completing-id>-retro.md` (v0)
3. `.claude/PRPs/retros/<completing-id>-retro.md`
4. `.claude/PRPs/retros/phase-<completing-id>-retro.md` (v0)

If the file is missing in all locations, or any of the three reflections is absent, **STOP the skill** and tell the user the outgoing advisor must write the retro first. Short-form is acceptable — even "nothing surprised us" under each is valid. Do not write the retro yourself; it's the outgoing advisor's reflection, not an orchestrator artifact.

**Four-role retro signals (advisory, not blocking).** v1 sub-phases run under the four-role model; the retro should reflect per-role signals — advisor / planning / impl / BM — not one single-session lump. See `feedback_four_role_retro_signals.md`. If the retro reads single-session-shaped on a four-role sub-phase, surface a one-line nudge: "this retro looks single-session-shaped — should the outgoing advisor revise with per-role signals before transition?"

**PMD-search-before-retro discipline (advisory, not blocking).** The carry-forward section is most valuable when it names cross-phase recurrences the outgoing advisor found via `memory_search_hybrid` over the brehon-fork PMD. The skill can't tell from the file whether a search was done; if the retro reads thin on cross-phase patterns and the completing phase had ≥3 notable bugs, surface a one-line nudge (not a stop).

Once the retro gate passes, read these to understand what happened and what comes next:

1. **Completing workflow-state record** — `workflow_state_<completing-slug>.md` in the brehon-fork PMD (auto-loaded via MEMORY.md, but re-read it explicitly here). The running decisions, commits, judgment calls from the completing phase. If it doesn't exist (the phase ran before this skill's brehon-fork-native rewrite, or under `/auto-phase` without a workflow-state file), note that and lean on the retro + the prior bootstrap file instead.

2. **Prior bootstrap file** — `.claude/PRPs/handovers/<completing-id>-bootstrap.md`. This is the structural-continuity source (it replaces the old "archived advisor-context"). Skim its section headers so the new bootstrap file keeps the same shape. If it doesn't exist (first transition under this skill), use the canonical reference shapes named in Step 4.

3. **Completing retro** — the file located by the retro gate. The three reflections are the outgoing advisor's signal. Carry-forward bullets → §3 of the new bootstrap file; change bullets → §5/§6. Preserve per-role structure if present.

4. **Completion report (if any)** — `.claude/PRPs/reports/<completing-id>-*-report.md`. Read for the impl agent's self-assessment.

5. **Next phase's plan or scope source** — if `.claude/PRPs/plans/<next-id>.plan.md` exists, skim it (do NOT read the full 500+ lines — section headers + §1 goal + §13 task list). If it doesn't exist, find the scope: for v0, `Grep` the next phase header in `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md`; for v1, the PRD (`.claude/PRPs/prds/<lane>.prd.md`) + the completing plan's "out-of-scope / deferred to -<next>" section (e.g. fed-in-a plan §19.4 lists the `-b` backlog). Note in the bootstrap file that the next session must author the planning brief → `/brehon-clarify` → queue planning Junior.

6. **Brehon-fork MEMORY.md** — read the "Active workflow state" section + skim the index. The lessons in `.claude/lessons/` are the durable cross-phase layer; the new bootstrap file references them by filename, never duplicates.

### Step 1 — Close the completing workflow-state record

The completing phase's workflow-state record in the brehon-fork PMD transitions from active scratchpad to historical CLOSED record. Resolve `workflow_state_<completing-slug>.md` in
`C:\Users\barri\.claude\projects\C--Users-barri-Developer-brehon-fork\memory\`.

1. If the file does not exist, **create it as a CLOSED record from scratch** (the phase ran under `/auto-phase` or pre-rewrite without one) — a short close record is better than none. Otherwise edit it in place.
2. Update the frontmatter: `name` includes "(CLOSED)" / "-SHIPPED-<date>"; `description` notes the ship date + PR # + merge SHA + final `governance-v0` HEAD; add "Historical close record. Read once during <next-id> session start for carry-forward. Do not act on the (now-complete) pipeline steps. Delete when <next-id> ships."
3. Add a `> **CLOSED.**` banner at the top of the body with: merge date+time, merge SHA, PR #, retro path, any lessons-promoted commit, any rule-revised commit. Mirror the canonical shape of `workflow_state_v1_ship_1_g4_replan_2026_05_18.md` (frontmatter + CLOSED blockquote + "What shipped" + "Root cause/fix if applicable" + "Validation evidence").
4. **Delete the two-phases-ago workflow-state record.** Keep only the immediately-prior CLOSED record live (it's read once at the next session's start). For a lane chain (e.g. fed-in-a → fed-in-b), the "two ago" is the CLOSED record from the phase before the completing one. Earlier records' durable content already lives in `.claude/lessons/` or the retros. If you can't unambiguously identify the two-ago record (lane changed, gap in the chain), skip the delete and note it — never delete a record you're unsure about.

### Step 2 — Create the fresh workflow-state skeleton

**Naming discipline (prevents `_new` filename pollution):** If a prior `workflow_state_<next-slug>.md` exists (e.g. an abandoned skeleton from a previous transition attempt), **delete it first**, then create the fresh skeleton with the canonical slug name. Never create a `workflow_state_<next-slug>_new.md` or any `_new`-suffixed variant — the PMD PostToolUse hook normalises the name immediately anyway, making the pointer in MEMORY.md stale from the first write. If a CLOSED record with that slug exists from a prior abandoned attempt, delete it and recreate cleanly.

Create `workflow_state_<next-slug>.md` in the brehon-fork PMD memory dir with this structure (substitute the resolved tokens; `<next-display>` is the human form, `<next-bootstrap-path>` is `.claude/PRPs/handovers/<next-id>-bootstrap.md`, `<completing-closed-record>` is `workflow_state_<completing-slug>.md`):

```markdown
---
name: workflow-state-<next-slug>
description: Mid-phase advisor running-state for <next-display> — <one-line phase goal>. Living scratchpad; survives session boundaries. Not yet started as of <today's date>.
metadata:
  node_type: memory
  type: project
---

# <next-display> — running state

## How to use this file

- Auto-loads via the brehon-fork `MEMORY.md` "Active workflow state" section.
- **Read `<next-bootstrap-path>` first** — that tracked in-repo file is the full handoff letter (reasoning texture + operational rules + session-start ritual). This memory holds the running facts; the bootstrap file holds the stable advice. If they conflict, the bootstrap file wins.
- **Read `<completing-closed-record>` once at session start** for carry-forward. Do not re-read.
- Write into the sections below as decisions land. Use absolute dates, never "yesterday".
- At <next-display> ship: this skill marks this file CLOSED and deletes the two-phases-ago record.

## <next-display> plan status

_Update this line only. Overwrite it as status changes._

**<today's date>:** <next-display> not yet started. Awaiting plan generation + advisor review. governance-v0 HEAD `<short-hash>`. Next action: author `.claude/PRPs/briefs/<next-id>-planning-1.md` → `/brehon-clarify` → queue planning Junior.

## Plan-shaping decisions

_Advisor decisions while reviewing the generated plan, before queueing planning/impl._

_(none yet)_

## Judgment calls during execution

_Scope reductions, ambiguity calls, trade-offs, advisor-answered DQ resolutions._

_(none yet)_

## Loop iteration observations

_Iteration count per task, polling-cycle anomalies, four-role transitions, surprises._

_(none yet)_

## Advisor watchlist during <next-display>

_Phase-specific risks beyond §4 of the bootstrap file. Add as new risks surface._

_(none yet)_

## Junior subagent learnings (harvest at retro)

_Lesson-worthy signals in Junior commits / DQ resolved entries. Harvested at retro → `.claude/lessons/` + PMD. See `feedback_junior_pmd_write_convention.md`._

_(none yet)_

## Deferred to later phase

_Items deliberately not addressed in <next-display>. Name the later phase + one-line reason._

_(none yet)_

## Blocked on external

_Items needing a human decision or external resource._

_(none yet)_

## Commits landed (<next-display>)

_Short hash + one-line summary per commit. Reconstruct the git chain without `git log`._

_(none yet)_

## Session handoff blocks (appended by /advisor-checkpoint)

_Most-recent block is authoritative on resume. Each: current focus, last decision, next expected artifact, tripwires, DQ pending, git-state snapshot._

_(none yet)_
```

### Pre-Step-3 — Stale bootstrap detection

Before creating the new bootstrap file, check if a prior bootstrap exists at `.claude/PRPs/handovers/<next-id>-bootstrap.md`:

```bash
ls C:/Users/barri/Developer/brehon-fork/.claude/PRPs/handovers/<next-id>-bootstrap.md 2>/dev/null
```

If it **does not exist**, proceed to Step 3 normally.

If it **does exist** (left over from a previous transition attempt), run:
```bash
git -C C:/Users/barri/Developer/brehon-fork log --oneline -1 governance-v0
# then compare to the governance-v0 HEAD recorded in the existing file's "Git state at handoff" block
```

If the existing file's governance-v0 HEAD differs from the live HEAD, the file is stale — it was written in a prior session before commits landed. **Do not trust its content.** Proceed to overwrite it in Step 3 using live `git rev-parse` output. Add a note to the transition commit body: "overwrite stale bootstrap from prior session (<old-hash> → <new-hash>)".

If the existing file's HEAD matches the live HEAD, the file was written this session — skip Step 3 (idempotent) or confirm with the user whether to regenerate.

### Step 3 — Write the merged bootstrap file

Create `.claude/PRPs/handovers/<next-id>-bootstrap.md`. **This is the single most important deliverable** — one tracked in-repo file that is both the reasoning texture (what used to be the homeserver advisor-context) AND the next-session entry point (what used to be the pasted bootstrap prompt). It must let a cold session resume with zero conversation context.

Mirror the canonical shape of the most recent existing bootstrap file (check `.claude/PRPs/handovers/` for the newest `*-bootstrap.md` — e.g. `v1-ship-1-bootstrap.md`, `v1-AD-e-resume-bootstrap.md` — and the structure used in `homeserver/.claude/advisor-context-v1-federation-inbound-b.md` which was the last advisor-context written before this rewrite; that 8-section body is the texture template).

**Structure (frontmatter + RESUME block + 8 sections + handoff data):**

```markdown
---
phase: <next-id>
plan: .claude/PRPs/plans/<next-id>.plan.md   # or "(not yet authored)"
phase_branch: phase-<next-id>                  # not yet created until bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-<lane>   # created at bm-cut; until then canonical brehon-fork
authored: <today's date>
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the <next-display> advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon <next-display>.** This is a fresh session (or a resumed one). The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-<lane>` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`). There is no homeserver session.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `<HANDOFF-HASH>` (see §"Git state at handoff"); if drifted, `git -C C:/Users/barri/Developer/brehon-fork log --oneline <HANDOFF-HASH>..governance-v0` and update your mental model before acting.
3. Read `.claude/decision-queue.json` (and the canonical resolver `scripts/brehon/resolve-dq-canonical.sh <next-id>` once a phase branch exists) for any pending entries since handoff; compare against the §"Decision-queue snapshot" below.
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_<next-slug>.md` is the running-state scratchpad (most-recent "Session handoff block" is authoritative on resume).

## Next concrete action

<one or two lines: e.g. "Author `.claude/PRPs/briefs/<next-id>-planning-1.md` (scope per <PRD/plan §X>) → `/brehon-clarify` → queue planning Junior." OR, if a plan already exists, the exact next stage per `.claude/rules/advisor-orchestrator.md` §3.1.>

---

## 1. <next-display> in one paragraph
[Task range, goal, what's new vs the completing phase, DoD.]

## 2. Why <next-display> is easier/harder than <completing-display>
[**Easier:** … / **Not easier:** … — concrete, not generic.]

## 3. Lessons from <completing-display> that apply to <next-display>
[Carry forward ONLY lessons relevant to the new domain. Reference
`.claude/lessons/feedback_*.md` by filename — never duplicate content.
For four-role sub-phases, separate by role: advisor-side / planning-side /
impl-side / BM-side. Pull from the completing retro's carry-forward +
per-role signals.]

## 4. <next-display>-specific watchlist
[3–6 watchpoints. Each MUST cite a specific file / table / schema line —
never just a concept (`feedback_advisor_watchpoint_specificity.md`). Each
states a forward gate ("plan §13 must …" / "the first test must assert …").]

## 5. Operational rules
[Polling cadence (~10 min, `mcp__junior-brehon__list_tasks`); brief discipline
(briefs in `.claude/PRPs/briefs/<next-id>-<role>-<n>.md`, committed to
governance-v0 first; pre-queue `memory_search_hybrid` + `/precheck` + §2.4
mandatory file-class lesson injection); LemmyResult Case A override for any
e2e brief; Shape G status (check DQ #229 / current state — SUSPENDED until
2026-06-01 means validate-pending-laptop per `advisor-orchestrator.md` §5.2);
Windows e2e bat-wrapper invocation + explicit-exit-marker reads; serial-cohort
discipline if hardware-constrained; model tiering (Planning→Opus, Impl→Sonnet,
BM/ci-watcher→Haiku); clarify gate; the 6 user gates; DQ attribution
(`chore|docs(advisor|decision-queue):`); memory headroom (no bulk e2e.rs
reads). Carry the completing phase's rules forward; adjust per the retro's
change bullets.]

## 6. What changed from <completing-display>'s rule set
[Explicit diff so the advisor knows what's new this phase.]

## 7. Catch-fire procedures
[Universal triggers from `.claude/rules/advisor-orchestrator.md` §5.5 +
phase-specific additions. Drop completing-phase-specific ones that no longer
apply.]

## 8. Archive after <next-display>
[The standard close: run `/brehon-phase-transition <next-id> <next-next-id>`.
This skill will: close `workflow_state_<next-slug>.md`, delete the two-ago
record, create the next skeleton, write the next bootstrap file, update
brehon-fork MEMORY.md, commit on governance-v0. The prior bootstrap file
(this one) stays in `.claude/PRPs/handovers/` as its own archive — git
history is the archive; no move.]

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `<short hash>` (captured <today's date>) — `<subject>`
- Phase branch HEAD: `<short hash or "not yet created (branch phase-<next-id> cut at bm-cut)">`
- Recent governance-v0 commits (`git -C C:/Users/barri/Developer/brehon-fork log --oneline -5 governance-v0`):

  ```
  <five lines>
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
<pending entries at handoff time, one per line: #id from=<role> kind=<kind> q=<first 80 chars>; OR exactly "(empty at handoff)">
<note any archived-but-still-relevant entries — e.g. a dated reminder that moved into a decision-queue-archive-*.json — so the next advisor re-surfaces it.>
```

## Stop-and-ask tripwires

[3–5 bullets, each starting literally `Stop and ask if:` and naming a concrete
condition (file / test / behaviour), not a vague concept. Written from the
completing phase's experience. Example shape: `Stop and ask if: the plan
introduces a new migration under crates/db_schema/migrations/** — this phase
is handler-only; a new migration is a scope violation.`]
```

**Content guidelines for the bootstrap file:**

- **Reference `.claude/lessons/feedback_*.md` by filename**, never duplicate. Say "see `feedback_clippy_test_style.md`", not the rule's content.
- **Every watchpoint cites a specific table/file/line.** The Phase 2b governance_log/public_case_log conflation is the cautionary tale (`feedback_advisor_watchpoint_specificity.md`).
- **The RESUME block + §0-equivalent session-start ritual is the load-bearing part** — a cold session reads it first and must be able to act from it alone. Keep it precise (exact hashes, exact next action).
- **Don't let texture accumulate.** Each bootstrap file is fresh for its phase. Reference the prior bootstrap file (it's in git) for history; don't paste history forward.
- **Git state is captured literally at Step 3 write time** — run `git -C C:/Users/barri/Developer/brehon-fork log --oneline -5 governance-v0` and `git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` immediately before writing the bootstrap file's "Git state at handoff" block, not earlier in the session. If commits landed between session start and Step 3, the earlier hash is stale. Do not paste a hash from memory or from the workflow-state record; always run the command fresh.

### Step 4 — Update the brehon-fork MEMORY.md index + commit

1. **Brehon-fork MEMORY.md** — edit only the "Active workflow state" section:
   - Update the top `RESUME:` line (if present) to the new current state (completing phase SHIPPED + the next action for `<next-id>`).
   - Add/replace the completing phase's line: mark `<completing-id> CLOSED — PR #N merged <date> (<sha>); retro signed off; transition complete <date>`.
   - Replace the "Active sub-phase" line with `<next-id>` + its one-line scope + "plan not yet authored" (or its stage if a plan exists).
   - Add a pointer line to the new bootstrap file: `[<next-id> bootstrap](handover path)` — note it's the tracked in-repo entry point.
   - Add the new `workflow_state_<next-slug>.md` pointer; update the completing record's pointer to "(CLOSED) … delete at <next-id> ship"; remove the deleted two-ago record's pointer.
   - Collapse stale per-slice lines into concise CLOSED summaries if the section is growing past its ~15-line budget (MEMORY.md lines after 200 truncate).

2. **Git housekeeping** (before staging):
   - Agent-worktree prune: `git -C C:/Users/barri/Developer/brehon-fork worktree list | grep '.claude/worktrees/agent-'` — for each: `git -C C:/Users/barri/Developer/brehon-fork worktree remove --force --force .claude/worktrees/<name>` (double `--force`; dead-PID locks refuse a single `--force`). If empty, skip.
   - `git -C C:/Users/barri/Developer/brehon-fork worktree prune`.

3. **Commit on brehon-fork governance-v0.** Stage the new bootstrap file (the workflow-state PMD files + MEMORY.md live in the brehon-fork PMD dir, which is OUTSIDE the git repo — they are not committed; they persist as memory). So the git commit contains only the in-repo artifact:
   - `git -C C:/Users/barri/Developer/brehon-fork add .claude/PRPs/handovers/<next-id>-bootstrap.md`
   - Subject: `chore(brehon): close <completing-id>, bootstrap <next-id>`
   - Body: one line on the workflow-state close + two-ago delete, one line on the new bootstrap file, one line on the retro location, the verified governance-v0 HEAD.
   - Push: `git -C C:/Users/barri/Developer/brehon-fork push origin governance-v0` (per `.claude/rules/phase-branch.md` — bootstrap/handover files are `.claude/` meta-work, direct-to-trunk, no PR).

   > **Note on what is and isn't version-controlled.** The bootstrap file (`.claude/PRPs/handovers/<id>-bootstrap.md`) is tracked in the brehon-fork git repo and committed. The workflow-state record + MEMORY.md are in the brehon-fork **PMD memory directory** (`C:\Users\barri\.claude\projects\C--Users-barri-Developer-brehon-fork\memory\`), which is NOT a git repo — they are durable memory, edited in place, not committed. This asymmetry is intentional: the handoff letter is an auditable repo artifact; the running-state scratchpad is session-spanning memory.

After the commit, **surface to the user (no pasted prompt — the bootstrap file IS the prompt):**

> Transition complete. Next session:
> 1. Open Claude Code in `C:/Users/barri/Developer/brehon-fork` (canonical, governance-v0) — or the `brehon-fork-<lane>` worktree once `bm-cut` creates it.
> 2. Read `.claude/PRPs/handovers/<next-id>-bootstrap.md` — start with its RESUME block.
> 3. Run its Session-start ritual, then the Next concrete action.
>
> [If the completing phase ran on a lane worktree:] Worktree cleanup — use the **absolute path** from `git worktree list`, not a relative path (relative paths can fail silently):
> ```
> git -C C:/Users/barri/Developer/brehon-fork worktree list   # read the exact registered path
> git worktree remove <ABSOLUTE_PATH_FROM_LIST>
> git branch -d phase-<completing-id>
> ```
> Do this only after confirming the PR merged + branch deleted on origin.

## Phase-specific considerations

- **Phase 2→3 (view crates → API DTOs):** View-crate rules (Selectable template, tuple-load) don't apply. DTO work is mostly mechanical. New: `ts-rs` export, `api_common` conventions, newtype-vs-raw-id.
- **Phase 3→4 (DTOs → endpoints):** Biggest jump — first business logic (handlers, transactions, permission checks). New: Actix-web handler conventions, transaction safety, the golden-path e2e.
- **Phase 4→5 (endpoints → reputation/sponsorship):** Reputation math, endorsement lifecycle, jury gating. New: floating-point scores, sponsor-liability cascading, jury picker.
- **Phase 5→6 (reputation → federation):** Outbound-only AP. New: federation protocol, activity serialization, instance trust policy.
- **v0 → v1 (Phase 7 → v1-AD-a or v1-JM-a):** Crosses the version boundary. Lane-based sub-phase ids; four-role orchestration via `.claude/rules/advisor-orchestrator.md` is the default.
- **v1 sub-phase → v1 sub-phase, same lane (e.g. v1-JM-c → v1-JM-d, or v1-federation-inbound-a → -b):** Most direct carry-forward — the lane's plan structure + PRD are stable; the completing plan's "deferred to -<next>" / out-of-scope section is the next phase's scope source. Watch for slice-specific schema deltas and DQ entries the prior slice deferred.
- **v1 lane → v1 lane (e.g. v1-AD-c → v1-JM-a):** Cross-lane. Carry-forward is structural (orchestration model, retro discipline, lesson corpus) not domain-specific. Treat like a Phase N→N+1 for domain purposes.

## Quality checks before committing

- [ ] Pre-flight CWD check passed: session is on the canonical `brehon-fork` checkout on `governance-v0` (not a lane worktree, not homeserver).
- [ ] Phase-id grammar applied: bootstrap filename uses display form; workflow-state PMD file uses slug form.
- [ ] Retro file located (one of the 4 resolution paths) with all three reflections present; four-role nudge surfaced if single-shaped.
- [ ] Completing `workflow_state_<completing-slug>.md` marked CLOSED (frontmatter + banner); two-phases-ago record deleted (or skip-noted if ambiguous).
- [ ] Fresh `workflow_state_<next-slug>.md` skeleton created with correct phase name + scope line + governance-v0 HEAD.
- [ ] Bootstrap file written: frontmatter + RESUME block + session-start ritual + 8 sections + git-state (literal, real output) + DQ snapshot + 3–5 `Stop and ask if:` tripwires.
- [ ] Bootstrap file watchpoints cite specific files/tables/lines, not concepts.
- [ ] No `.claude/lessons/feedback_*.md` content duplicated into the bootstrap file (filename references only).
- [ ] Bootstrap file references the prior bootstrap file for structural continuity (it's the archive — in git history).
- [ ] Brehon-fork MEMORY.md "Active workflow state" updated; no stale pointer to the deleted two-ago record; section within its ~15-line budget.
- [ ] Commit contains ONLY `.claude/PRPs/handovers/<next-id>-bootstrap.md` (workflow-state + MEMORY.md are PMD memory, not git); subject `chore(brehon): close <completing-id>, bootstrap <next-id>`; pushed to governance-v0.
- [ ] No homeserver path was read or written (per the Migration note — homeserver artifacts are frozen).
- [ ] **Entry-kind count cross-check (ONLY if the completing phase registered new `ENTRY_KIND_*` consts):** re-derive `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` and confirm BOTH the headline count AND the era-breakdown sum in `docs/brehon-law-inspired-network/04-data-model-and-api.md` §11 match it (fixing only the headline leaves the breakdown drifting — the two-layer drift). The authoritative source is `.claude/rules/governance-log-entry-kind-registry.md`; `04` §11 is the derived mirror. If `04` §11 is stale, fix it in this transition's commit (it's `.claude/`-adjacent meta-work, direct-to-trunk). Per `feedback_entry_kind_runtime_allowlist_check.md` + the 2026-06-18 m3-entry-kinds retro Change #2. Skip this line entirely for non-entry-kind phases.
- [ ] Closing surface given to user: open-in-brehon-fork + read-bootstrap + ritual; lane-worktree cleanup line if applicable.

## Pre-commit dogfood (per `.claude/rules/advisor-orchestrator.md` §3.7)

This rewrite was walked through against the real **v1-federation-inbound-a → v1-federation-inbound-b** transition (completed 2026-05-19, the transition immediately preceding this rewrite).

- **What worked:** the four-deliverable mapping holds against that transition's actual artifacts. The completing retro (`v1-federation-inbound-a-retro.md`) was the 8-section four-role shape — the relaxed retro gate (accept the 8-section mapping, not just literal H2s) correctly admits it. The brehon-fork MEMORY.md already had an "Active workflow state" section with `workflow_state_<slug>.md` pointers and a `RESUME:` line — Step 4 edits exactly that, no new structure invented. `.claude/PRPs/handovers/` already contained `v1-ship-1-bootstrap.md` + `v1-AD-e-resume-bootstrap.md` with the frontmatter+RESUME+sections shape — Step 3's template mirrors a real precedent, not an invented one. The "commit only the in-repo file; PMD memory is not git" asymmetry matches what actually happened (the fed-in-a homeserver-side commit touched only `.claude/`; the PMD `.md` files were never in any repo).
- **What didn't (and is now fixed):** the old skill's Step 1 (git-mv the homeserver advisor-context to `homeserver/.claude/archive/`) had no brehon-fork equivalent and would have errored — removed; the prior bootstrap file is its own archive via git history. The old Step 5 produced a 300–500-word pasted prompt for a "homeserver session" that no longer exists — folded into Step 3's tracked file + a 3-line closing surface. The old Step 6 updated the homeserver `project_brehon_governance_platform.md` phase table — dropped; brehon-fork has no equivalent platform-memory file, the MEMORY.md "Active workflow state" section carries that signal.
- **Residual gap noted:** the fed-in-b advisor-context the prior transition wrote lives homeserver-side (`homeserver/.claude/advisor-context-v1-federation-inbound-b.md`). Per the user's "leave homeserver frozen" decision, this skill does not migrate it. The fed-in-b advisor reads that homeserver file once at fed-in-b start; the **fed-in-b → fed-in-c** transition is the first to run fully brehon-fork-native (it will write `.claude/PRPs/handovers/v1-federation-inbound-c-bootstrap.md` and `workflow_state_v1_federation_inbound_c.md` with no homeserver touch). This is expected, not a defect.
