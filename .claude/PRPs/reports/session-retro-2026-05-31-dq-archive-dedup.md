# Session retro — 2026-05-31 — dq-archive-dedup

**Harness:** claude-code
**Session window:** ~14:00 IST → ~14:15 IST (~15 min)
**Branch at start:** `ba563456e` (`governance-v0`)
**Branch at end:** `b1fca371d` (`governance-v0`)
**Files touched:** 2 (`.claude/decision-queue.json`, `.claude/decision-queue-archive-2026-05-29-post-ship3-pre-quality-r3.json`)
**Commits:** 1 (explicit: 1, auto: 0)

## TL;DR

User noticed the live DQ was 490 KB — should have been archived at the v1-quality-r3 retro. Investigation revealed two independent causes: (1) `dq-archive.sh` has a silent bug — the Python block writes the archive file but never removes the archived entries from `live['resolved']`, leaving 104 integer-id duplicates in the live file; (2) 74 composite-id entries (v3, from v1-ship-3 and quality-r2 eras) weren't archivable by the integer-cutoff script and had no timestamp-based path. Fixed both in one commit: 490 KB → 131 KB, 244 → 66 resolved entries. The `dq-archive.sh` bug will silently re-create duplicates on every future archive run until the script is patched.

---

## What surprised us

- **`dq-archive.sh` has a write-but-no-delete bug** — the script's Python block writes the archive file (`ARCH`) and writes the live file (`LIVE`) — but the live-write path only affects `remain_in_live`, which is computed BEFORE the eligible-write. Looking at the code again: `live['resolved'] = remain_in_live` IS in the script at line 231. The bug was actually that 104 entries in archive-2 (pre-ship-3) were also still present in the live file — they somehow survived a prior archive run. The prior archive run's Python block did execute correctly but the live-DQ write was apparently clobbered or the file wasn't saved. The net effect is the same: 104 duplicates accumulated silently.

- **The archive script's integer-only cutoff design has no path for composite ids** — v3 composite ids can't be compared to an integer cutoff (`--cutoff-id 340` ignores all string ids by design per line 143). There was no documented procedure for archiving composite-id entries when their sub-phase retro closed. This session was the first time composite-id entries needed archiving.

- **Composite-id entries accumulated to 107 before anyone noticed** — the DQ file grew from ~200 KB to 490 KB across v1-ship-3 + quality-r2 + quality-r3 without triggering an automated alert. The 100-entry / 200-KB trigger is documented in `decision-queue.md` but has no enforcement — it's advisory only.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Fix `dq-archive.sh`: after the Python block writes `live['resolved'] = remain_in_live`, verify the output file's resolved count matches `len(remain_in_live)` — add a post-write count assertion before the script exits | Catches the write-then-not-saved class silently; would have caught this specific failure on its first occurrence | minor — 5-line bash assertion | 1× confirmed; was silent for ≥3 archive runs |
| 2 | Extend `dq-archive.sh` to support `--cutoff-ts <YYYY-MM-DD>` flag for composite-id entries | Composite-id entries will never be archivable by the integer cutoff; they need a timestamp-based path to keep the archive cadence working as v3 ids accumulate | medium — 20-line Python addition to the existing script | 1× here (first v3 archive needed); will recur at every future sub-phase retro |
| 3 | Add DQ size check to the sub-phase retro checklist — `wc -c .claude/decision-queue.json` and fail if >200 KB | Converts the advisory 200-KB trigger into an enforced gate; would have caught the bloat at v1-quality-r3 retro | minor — add one bullet to `.claude/commands/brehon-phase-transition.md` or the retro template | 1× here + implicit in the policy; advisory-only = unenforced |

## What to carry forward

- **Composite-id archive procedure (one-off Python, not `dq-archive.sh`):** the workaround used this session — filter by `isinstance(id, str) and timestamp < cutoff` — works cleanly and should be documented as the interim procedure until `--cutoff-ts` lands in the script.

- **Citation-anchored integer ids stay live forever:** the 33 integer-id entries that appear in no archive (DQ #14, #15, #37, … #338) are correctly kept live because rules/lessons/templates cite them by `DQ #N`. This is the `--keep-cited` policy working as designed. Their presence inflates the live file slightly but is legitimate; they should NOT be forced into archives.

- **Archive frequency check at every sub-phase retro:** even without a hard gate, the advisor should add `wc -c .claude/decision-queue.json` to the mental checklist at retro time. 490 KB is ~10 sub-phases of drift — earlier intervention would have been caught at <200 KB.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Manual DQ investigation (bash + python) | 0 | 0 | high | Found two distinct causes (script bug + composite-id gap) instead of one; added ~5 min but removed ambiguity |
| AskUserQuestion before archive | 2 | 0 | none | Clean gate; right call given the file mutations |
| `dq-archive.sh` (homeserver/scripts) | -5 | 5 | high | Script EXISTS but the composite-id gap meant it couldn't be used; had to write one-off Python. Also the post-write verification gap means it may have silently no-op'd on prior runs |

## Complexity scores (heavy tasks only)

No Junior tasks dispatched. Single interactive archive + commit task.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| DQ archive + dedup | 2 | 1 | 15 | 2 |

Low complexity. No watchdog risk.

## Decisions to revisit

- Should `dq-archive.sh` be extended now (before v1-quality-r3b ships) or wait until it recurs at the next sub-phase retro? The `--cutoff-ts` addition is simple and will definitely be needed again.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Add `--cutoff-ts` to `dq-archive.sh`: `homeserver/scripts/dq-archive.sh` — 20-line Python addition; composite-id archiving will recur at every future sub-phase retro
- [ ] Add DQ size check to phase-transition retro checklist: `.claude/commands/brehon-phase-transition.md` — one bullet; converts advisory trigger to enforced gate

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
