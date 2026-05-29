# Handover — harness context-budget redesign (2026-05-29)

**For:** the next advisor session resuming this work.
**Branch:** `governance-v0` (canonical `brehon-fork` checkout — meta-edits only; do NOT touch phase branches).
**Read with zero conversation context — everything you need is below.**

---

## RESUME BLOCK

### The goal (user-set)

Cut the Claude Code **always-load Memory-files baseline from ~71k tokens → as close to ≤22k as the Pi dual-harness contract allows**. Total session baseline target ≤50k (fixed non-memory load ≈28k). The deep audit proved **≤22k is structurally impossible** (Pi-citation freeze map + irreducible cross-harness DQ-schema/gate-safety policy); the **defensible floor is ~50k**, reachable via a ~17–20k structural relocation. User accepted "Execute B1+B2+B3 + MEMORY rewrite (hit ~50k)".

### Authoritative documents (read these first)

1. `.claude/PRPs/reports/harness-redesign-session-profiles-2026-05-29.md` — **THE plan.** §2 = the exhaustive Pi-citation freeze map (which `##`/`###` headings are FROZEN vs relocatable). §3 = relocation plan with token deltas (B1/B2/B3). §5 = why ≤22k is impossible + the ~50k floor justification. **The freeze map in §2 is the gating constraint for every edit.**
2. `.claude/PRPs/reports/harness-audit-2026-05-29.md` — predecessor conservative-trim audit (applied `f9982bd6e`, ~1.8k won). Its "Execute-pass actuals" section has the **load-bearing token-math correction: live tokens ≈ chars × (1/2.4), NOT chars/4.** Use 2.4 chars→token for all markdown estimates.

### What's DONE (committed on governance-v0 this session)

| Commit | What |
|---|---|
| `bb0846683` | **Guard shipped** — `scripts/brehon/verify-rule-anchors.sh` (the safety net). Scans every cross-harness section-name citation of the 5 dual-harness rule files; verifies each cited heading resolves. Also restored the `### When to override` stub in advisor-orchestrator.md §5.1 (pre-existing dangle). |
| `7b7a2ab98` | **B1 COMPLETE** — relocated advisor-orchestrator.md §4 (cohort dispatch) + §5.3 (§G4 classifier tables) + §2.1 (brief-location) MECHANISM to `refs/auto-phase.md`. Kept all frozen heading stubs + pointers. advisor-orchestrator.md: 47,410 → 33,207 chars (−5,900 tok). |

### What REMAINS (do these in order)

- **B2 — decision-queue.md → refs** (`~3,900 tok`). Move to `refs/dq-mechanics.md` (exists) + `refs/dq-recipes.md` (exists):
  - `§Polling-loop routing per kind` (advisor-only mechanism — 0 Pi cites)
  - `§Next-id calculation (pre-v3 — historical)` + `§Deprecated kinds + two-phase Shape-G` (pure history — already have refs pointers)
  - `§Recipes (copy-pasteable)` (bodies already in refs/dq-recipes)
  - `§Concurrency` + parts of `§Subagents and attribution` that overlap `§Attribution integrity`
  - `§Archive policy` detail + `§What does NOT get archived`
  - **FROZEN — DO NOT MOVE (the gating constraint):** `§ci-watcher mutation pattern (option 2)` (**11× Pi citations from .pi/skills/ci-watcher — the hardest anchor in the corpus**), `§Attribution integrity` (6× cited), `§Mid-task visibility` (2× Pi), `§Schema (v3)`, `§kind: blocker/log/clarify/validate-pending` (2× Pi). Keep these resident verbatim.
- **B3 — multi-lane-worktree.md → refs/multi-lane-mechanics.md** (`~4,170 tok`, exists). multi-lane has **0 Pi citations** — least constrained. Move: `§Lane modes / Mode A / Mode B`, `§Brief location and trunk→phase sync` (SSH recipe), `§Daemon side`, `§Migration plan`, `§Why this rule exists`, `§PMD is cross-lane shared` (overlaps pmd-invariants #1). **Keep resident:** `§Layout`, `§Hard refusals`, `§Lifecycle` (Claude-cited 3×), `§Session-start ritual` (Claude-cited).
- **MEMORY.md structural rewrite** (`~2-4k tok` + it's currently ~26,318 bytes = OVER the 24.4 KB budget). Move CLOSED-lane detail to topic files, keep one-line active pointers. This is a `memory-prune` structural pass. Bytes bind first.

### The METHOD (follow exactly — it's how B1 stayed safe)

1. For each chunk: read the section in the rule file, append it (verbatim, with a relocation provenance note) to the destination refs file, then shrink the rule section to **frozen-heading-stub + terse statement + pointer to the refs §**.
2. **Keep every FROZEN heading as a heading** (per §2 freeze map). Pi cites headings by prose name — moving/renaming one breaks Pi silently.
3. **Keep SAFETY policy resident, move only MECHANISM.** (B1 kept §3.2 gates, cycle-count meta-rule, catch-fire table, DQ triage tree resident; moved only the step-by-step tables.)
4. After each rule file's edits: run `bash scripts/brehon/verify-rule-anchors.sh --list`. **The baseline has exactly 3 known-benign dangles** (compound-name/phrase artifacts, NOT real breaks): `decision-queue.md §"Attribution integrity §Detection"`, `decision-queue.md §"Recipe 2 self-resolved"`, `multi-lane-worktree.md §"multiple active files"`. **Any NEW dangle = you broke a citation; fix before committing.**
5. Commit each B-step separately with measured char/token delta in the commit body.

### VALIDATION (user's instruction — do this in the new session)

After B2+B3+MEMORY land, **validate the cut against the baseline via `/context`**:
- **Baseline (this session start):** Memory files = **71k tokens (7.1% of 1M / ~35% of 200K effective)**. The 16 always-load entries are listed in the redesign report §0.
- **Target after all moves:** Memory files ≈ **50–54k tokens**. Run `/context` in the NEW session (fresh load picks up the relocations) and compare the "Memory files" line. Note: `/context` itself inflates "Messages" — read the **Memory files** bucket specifically, not total.
- Expected per-file: advisor-orchestrator ~13k (was 19.8k), decision-queue ~8k (was 12.3k), multi-lane ~3k (was 7.3k), MEMORY.md ~7-8k (was 11.4k).

### Tasks (TaskList state at handover)

- #1 Read report + freeze map — **completed**
- #2 B1 advisor-orchestrator — **completed**
- #3 B2 decision-queue — **pending** (next)
- #4 B3 multi-lane — pending
- #5 MEMORY.md rewrite — pending
- #6 Guard — **completed**

### Cross-session notes / hazards

- **Lane topology:** `git worktree list` shows `brehon-fork-redaction-r1` (phase-v1-redaction-r1) + `brehon-fork-rt-r4` (phase-v1-RT-r4) active. This work is governance-v0 meta-edits in canonical only — no phase-branch contact. No conflict, but other sessions may be driving those lanes.
- **Stray deletion (NOT mine, NOT addressed):** `.claude/PRPs/plans/v1-data-model-doc-and-04-retire.plan.md` is deleted in the working tree (was committed in `96b86a8de` this session, then removed). Left untouched — may be intentional cleanup from the data-model-doc consolidation. If accidental: `git checkout -- <path>`. Surfaced to user; unresolved.
- **Active Brehon work this session (do not disturb):** v1-quality-r2 Task 0 (#509) was running per the cold-resume runlog; v1-RT-r4 just cut. This redesign is orthogonal infra work.
- **The guard is now wireable as a PreToolUse hook** on rule edits (future) — currently standalone. Run it manually after each relocation.

### Honest framing to carry forward (from the audit)

The one-time structural win is **~17-20k tokens (10× the conservative trim's 1.8k)**, landing at **~50-54k**, NOT 22k. The durable lever is **growth-discipline** — new mechanism/incident-narrative authored in `refs/` from the start (the `rule-narrative-bloat-reminder.sh` PostToolUse hook + advisor-orchestrator §3.6 already enforce this). The `verify-rule-anchors.sh` guard now makes the Pi-freeze map enforced, not just documented.
