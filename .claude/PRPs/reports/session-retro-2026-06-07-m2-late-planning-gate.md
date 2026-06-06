# Session retro — 2026-06-07 — m2-late planning gate

**Harness:** claude-code  
**Session window:** ~2026-06-06 22:00 UTC → 2026-06-07 ~03:00 UTC (~5 h)  
**Branch at start:** `0001895f9` (`governance-v0`)  
**Branch at end:** `7f4710c31` (`governance-v0`)  
**Files touched:** ~15 across 7 commits this session window (plus prior-window governance-fix and m2-rooms-a close)  
**Commits (this window):** 7 (all advisor-authored; no Junior tasks completed this window — #639 dispatched, still running)

## TL;DR

Session bridged the m2-rooms-a close (governance bug fix cycle + bm-merge) through the m2-late planning gate. The most load-bearing finding: **the cross-repo sync tool silently reverted a shipped context-budget trim** — 4 rule files deleted on 2026-05-31 (universal-guards.md consolidation) reappeared 3 days later via a sync auto-commit, adding ~2.6k tokens of duplicate always-load content. The fix was a harness-regression-guard hook + tombstone ledger. Top change for next session: wire the tombstone ledger update into the harness-audit P1 flow so future rule deletions auto-register their guard.

---

## What surprised us

**Advisor:**
- **Silent sync regression (3 days undetected).** The 2026-05-31 `universal-guards.md` consolidation was a deliberate P1 context trim. The homeserver `sync-shared-skills.sh` re-committed all 4 deleted files on 2026-06-04 because the script lacked a per-repo exclusion list. Zero alarms fired. The audit retro from this session (`d8e8b146b`) only caught it because a human-run harness-audit was in scope; routine advisor sessions would have re-loaded duplicate content forever. **The structural surprise is that a shipped trim has no persistence guarantee across the sync boundary.**
- **SanctionAction variant count off-by-one in the brief.** Brief §2 stated "Seven variants" but `crates/db_schema_file/src/enums.rs:560-577` has 8. The clarify pass (`/brehon-clarify`) caught this before the planning Junior ran — saving at minimum one plan revision cycle. The surprising element: the brief author (advisor, prior session) mis-counted without reading the file first. The canonical-schema-first gate (`feedback_read_canonical_before_writing_spec.md`) applies here — a 30-second `grep` at brief-author time would have caught it.
- **Planning Junior (#639) was already running at session start.** The prior-context-window session dispatched it before the compact. Arrived into this session with the planning task live and nothing to do about it. Clean handoff overall, but the `/precheck` was run anyway — confirming the advisor's pre-queue ritual is harmless when the task is already in-flight.
- **med-fix brief had competing DoD instructions (3rd recurrence).** `governance-v0-fix-impl-med-1.md` initially had both "DoD per commit: run cargo check" and "write validate-pending-laptop DQ then stop." Prior session had to delete the cargo DoD. This is the promoted pattern (ID 864): "impl-task brief must never include per-commit cargo DoD when validate-pending-laptop is the gate." 3rd occurrence confirms the pattern is load-bearing but the structural fix (brief template audit) is still pending.

**Planning:**
- N/A (planning task #639 ran across session boundary; signals will appear in the post-planning approval session).

**Impl:**
- N/A (governance-fix impl tasks completed in prior window).

**BM:**
- PR #191 (m2-rooms-a) was already merged by prior session when `/bm-merge` was invoked. The bm-merge script correctly identified `state: MERGED` and stopped. No wasted work — the Phase 8.5 tombstone for the m2-rooms-a bootstrap was still needed and applied.

---

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **harness-audit SKILL.md Phase 4 delete step: require tombstone ledger append** (done in `7f4710c31`) | Future rule deletions auto-register in `.claude/refs/harness-deleted-rules.txt`; the regression guard fires the next time sync reverts them | minor — already shipped | 1× this session (novel but structurally inevitable) |
| 2 | **Fix `sync-shared-skills.sh` on homeserver to respect per-repo RULE_EXCLUDE list** | Sync tool no longer blindly overwrites repo-specific rule deletions; the `RULE_EXCLUDE` mechanism prevents the revert | medium (homeserver scripts) | 1× observed; root cause addressed per retro `d8e8b146b` |
| 3 | **Add `canonical-schema-first` gate to brief-author checklist: grep the enum/table before writing variant count** | Catches off-by-one errors like the SanctionAction 7→8 before the clarify pass; ~30s check | minor | 1× this session, but `feedback_read_canonical_before_writing_spec.md` already covers it — the gap is the brief-author doesn't run it against enums specifically |
| 4 | **Structural fix for the competing-DoD brief pattern (ID 864)**: audit `.claude/PRPs/templates/plan.template.md` Task template block — remove "DoD per commit: run cargo check" row entirely from the template when `validate-pending-laptop` is the gate. Add an explicit "⛔ DO NOT run cargo on daemon" constraint to the impl-task brief template §4 | Prevents the pattern from being authored into new briefs; the lesson exists but the template still has the footgun | medium | 3× confirmed (pattern ID 864) — **threshold met for structural fix** |
| 5 | **Recreate Telegram completion hook at session start** (hook #1 absent — daemon restart wiped it) | Session would have received Telegram notification when #639 completes instead of requiring manual polling | minor | recurring every daemon restart; see `feedback_daemon_telegram_completion_hook.md` |

---

## What to carry forward

- **The tombstone ledger pattern**: any time a rule or lesson is intentionally deleted (not just temporarily removed), append its basename to `.claude/refs/harness-deleted-rules.txt`. The guard hook reads it at SessionStart. This makes the deletion durable across sync boundaries.
- **Clarify-before-plan discipline still paying off**: the SanctionAction variant catch in `/brehon-clarify` is a clean example of the skill doing exactly what it was designed to do — catch a wrong premise in the brief before the planning Junior spends tokens on a plan that names 7 variants where 8 exist. The clarify pass on m2-late took ~10 min and saved at minimum one plan-revision cycle.
- **Post-merge DQ duplicate cleanup**: when merging a worker branch via `--no-ff`, the worker's `pending[]` DQ entries land in the merged tree alongside the trunk's `resolved[]` copy. Immediate Python cleanup required. Two occurrences in prior window — already promoted to lesson; carry forward the awareness.
- **Daemon-first finalize check**: for finished Junior tasks, check daemon-local first (`ssh homeserver "git log governance-v0 -1"`) before assuming origin is authoritative. Saves multi-probe diffs when daemon hasn't pushed yet.
- **`bridge_room.rs` untracked in canonical worktree**: `services/bridge/src/bridge_room.rs` shows as untracked in the canonical checkout. Verified it's committed on the phase branch (m2-rooms-a shipped it). Safe to ignore in canonical — the file exists on the merged phase branch tip, which is now on `governance-v0`. No action needed; just don't stage it here.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/brehon-clarify` on m2-late-planning-1 | 30 | 0 | low (worked as designed) | Caught SanctionAction 7→8 off-by-one; DQ `a3d0e9941441-057` resolved self |
| validate-pending-laptop cycle (governance-fix-high + med) | 0 | ~25 | medium | Per-commit cargo DoD conflict in med brief forced brief rewrite + re-dispatch; prior window; 3rd recurrence |
| `harness-audit` skill (manual run) | 45 | 0 | high | Found sync regression not expected; ~2.6k token savings per session going forward |
| `/bm-merge` PR #191 | 0 | ~3 | low | Already merged; bm-merge correctly stopped; no wasted work |
| OQ resolution (ADR016-02 + 04) | 20 | 0 | none | Clean resolvability check; unblocked m2-late planning gate |
| m2-late planning brief authorship | 15 | 0 | none | Smooth; 332 lines; all lesson injections fired correctly |
| Debug artifact cleanup | 2 | 0 | none | 7 stale files removed |

---

## Complexity scores (heavy tasks only)

No impl tasks completed in this session window. Governance-fix tasks completed in prior window:

| Task | Files | Commits | Runtime (min) | Max log silence (min) | Notes |
|---|---:|---:|---:|---:|---|
| governance-fix-high-1 (#634, failed) | — | 0 | ~350 | >60 | Watchdog kill — per-commit cargo DoD competed with validate-pending-laptop gate |
| governance-fix-high-1 re-dispatch (implied prior) | 3 | 3 | ~15 | — | Clean after brief fix |
| governance-fix-med-1 (#636) | 4 | 3 | ~12 | — | Clean after brief fix removing cargo DoD |
| m2-rooms-a cr-fix-1 (#638) | 3 | 2 | ~8 | — | Bridge config fixes; clean |

Task #634 is the outlier: 350 min runtime, watchdog-killed at exit 143, competing DoD instructions. Confirm-3rd-recurrence of pattern ID 864 — structural brief template fix warranted.

---

## Decisions to revisit

- **`bridge_room.rs` untracked in canonical**: verify once on governance-v0 post-merge that `git ls-files services/bridge/src/bridge_room.rs` returns the file. The merge of m2-rooms-a should have brought it in.
- **Telegram hook recreation cadence**: hook #1 absent at every session start after a daemon restart. Consider a SessionStart hook that auto-checks `list_hooks` and auto-creates if absent — but this has a side effect of auto-firing outbound Telegram on session boot. Revisit.
- **planning Junior #639 cross-session handoff**: next session must run full plan approval gate (§3.4 DoD smoke + §3.5 watchpoints) before queuing impl. The governance-log kind registry update (`sanction_published`, `sanction_event_delivery_failed`) must also be authored before impl dispatch.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Competing-DoD structural fix** (pattern ID 864, 3× recurrence): audit impl-task brief template §4 block to remove "DoD per commit: run cargo check" when validate-pending-laptop is the gate. Update `.claude/PRPs/templates/plan.template.md` and the impl-task brief template.
- [ ] **Tombstone ledger discipline**: promote the `harness-regression-guard.sh` + tombstone ledger as a cross-phase pattern to `feedback_harness_audit_tombstone_ledger.md` so future harness-audit sessions know to update it.
- [ ] **Canonical-schema-first for enum variant counts**: add a specific mention to `feedback_read_canonical_before_writing_spec.md` that enum variant counts must be verified by reading the source enum before authoring a brief that names a count.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
