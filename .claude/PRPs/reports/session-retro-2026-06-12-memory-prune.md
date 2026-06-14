# Session retro — 2026-06-12 — memory-prune

**Harness:** claude-code
**Session window:** ~2026-06-12 16:00 → ~16:45 IST (~45 min)
**Branch at start:** `610581d2a` (`governance-v0`)
**Branch at end:** `610581d2a` (`governance-v0`) — no commits this session
**Files touched:** 1 (MEMORY.md only)
**Commits:** 0 (skill output is not git-committed; MEMORY.md is in user-scope, not repo)

## TL;DR

Session ran `/memory-prune` on a MEMORY.md at 33,002 bytes — 8,602 bytes over the 24,400-byte hard budget. The file had been truncating at every SessionStart. Two write passes and ~15 targeted single-line Edit calls got it to 24,396 bytes (under by 4 bytes). The main surprise was how difficult it was to land exactly under the byte ceiling: two full-file Write attempts left the file still over, requiring ~12 subsequent micro-trim Edits to close the gap. The change proposal: add a "bytes-remaining buffer" to the prune target — aim for 23,800 (600-byte margin) rather than 24,400 exactly, so rounding errors in archive lines don't leave the file over.

---

## What surprised us

- **The archive line byte cost is non-trivial but easy to underestimate.** The skill's own guidance warns about this, but even after reading it, the 2026-06-12 archive line for 7 retired entries consumed ~140 bytes — nearly the entire margin from the second Write pass. Landing the final file at exactly 24,396 required 12 additional single-character Edit calls after the two bulk Write passes. The guidance says "budget for the archive line" but doesn't quantify it; in practice a 7-entry archive line is ~140 bytes.

- **Full-file Write + wc-verify loop is the correct strategy, but it burns 2× the Write calls vs. targeted Edits.** The first Write pass removed the major items but still landed at 28,944 bytes — a second full Write was needed. 28,944 → 24,758 → 24,396 required TWO full rewrites plus ~12 Edit calls. A tighter pre-estimate of the target size (e.g. compute line-lengths before writing) would reduce this to 1 full Write + ~3 Edits.

- **The Edit tool's `old_string` matching failed on the first attempt** (line 18, the m2-core-hook entry) because the file uses leading tabs in list items (`\t-`) but the Edit `old_string` used spaces. The fallback — a full-file Write — was correct but cost an extra operation. The skill's "use Edit, not shell heredocs" guidance is sound; the trap is not verifying the exact leading whitespace before constructing the `old_string`.

- **220 lines / 24,396 bytes — lines are well under 200 line soft ceiling but bytes were the binding limit throughout.** Consistent with the skill's documented observation. The skill correctly gates on bytes first.

---

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Prune target: aim for 23,800 bytes, not 24,400.** Add an explicit note to the skill Step 1 urgency gate: "Target: prune to ≤23,800 (600-byte margin for rounding and archive line)" — the 4-byte final margin is dangerously close to the limit. A single line with an extra word next session tips it over. | Eliminates the "19 bytes over" → "65 over" → "0 over" micro-trim loop | minor (1-line skill edit) | 1× this session; implied by the skill's own archive-line caveat |
| 2 | **Pre-estimate target size before the first Write.** Before the full-file Write in Step 4, compute: `(current_bytes - bytes_to_remove + archive_line_estimate)` and check < 23,800. Even a rough estimate (sum `wc -c` of lines to remove, subtract, add 150 for archive) would have shown "still need 200 more bytes" before the second Write. | Reduces from 2 full-Write passes to 1 | minor (mental arithmetic step, no tool change) | 1× this session |
| 3 | **Add a leading-whitespace guard to Step 4 Edit instructions.** The existing "copy the exact surrounding text" note is correct but doesn't warn about tab-vs-space in list items. Add: "MEMORY.md list items use `- ` (hyphen+space) with NO leading tab in index entries; verify with `cat -A` or `head` before constructing `old_string`." | Prevents the Edit-fail→full-Write fallback for leading-whitespace mismatches | minor (1 sentence in skill body) | 1× this session; generic Write-vs-Edit trap |

---

## What to carry forward

- **Write tool is the right call when ≥5 entries change.** Attempting to Edit 17 entries individually would have been slower and more error-prone. The full-file Write strategy (two passes) was correct; the only improvement is the pre-estimate to reduce passes.
- **Section-removal + shortening + Historical-append in one Write pass is the efficient shape.** Trying to do them as separate Edit calls would have taken 3× as long. Write once, measure, micro-trim with Edits.
- **Milestone-transfer-test was the right discriminator.** The 7 entries moved to Historical all passed a clear test: "strip the phase citation — does anything remain?" For pi-harness-alignment and planning-001 results, nothing remained. For all promoted patterns, phases are just incident evidence — the pattern transfers.
- **Absolute-path links (`../../Developer/brehon-fork/...`) in MEMORY.md cost ~50 extra bytes vs. bare filenames.** Shortening them to bare filenames (since the links resolve relative to the memory system's CWD) reclaimed ~150 bytes. Worth doing systematically on any lesson entry that uses the full relative path.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/memory-prune` skill | 25 | 12 | medium | Saved: systematic classification + Write-vs-Edit guidance; Wasted: 12 min on micro-trim loop after second Write still over. Surprise: Edit `old_string` mismatch on first call (tab/space trap). |
| `wc -c / wc -l` measurement cadence | 5 | 2 | none | Correct tool, used appropriately at each step. |
| Full-file Write strategy | 15 | 8 | low | Two passes needed vs expected one; pre-estimate would have caught this. |

## Complexity scores (heavy tasks only)

This session had no Junior tasks or impl-task dispatches. The single "heavy" operation was the MEMORY.md prune:

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| MEMORY.md prune (33KB → 24.4KB) | 1 | 0 | ~45 | ~8 |

No watchdog risk; complexity was medium due to iterative byte-measurement loop.

## Decisions to revisit

- The 24,400 byte budget appears to be measured at load time with CRLF/LF sensitivity on Windows. If the file is saved with CRLF line endings, the byte count is higher than wc-c reports. Worth verifying that the harness measures the same way wc-c does (LF-only).

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Prune target 23,800 buffer:** update `.claude/skills/memory-prune/SKILL.md` Step 1 urgency gate to add "Target: ≤23,800 to leave margin for archive line" — minor edit, zero risk
- [ ] **Pre-estimate before Write:** add Step 3.6 in the skill: compute (remove_bytes - archive_line_estimate) before deciding if one Write suffices or two are needed — minor addition
- [ ] **Absolute-path shortening:** add to Step 2 classification table a new row: "Verbose absolute path" — `../../Developer/brehon-fork/.claude/lessons/` prefix on lessons links → shorten to bare filename — saves ~50 bytes per entry

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
