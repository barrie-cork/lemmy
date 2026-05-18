# Session retro — 2026-05-16 — pmd-backfill-dq-hook-scope

**Harness:** claude-code
**Session window:** ~19:45 UTC → ~21:30 UTC (~105 min)
**Branch at start:** `d466d93d7` (`governance-v0`)
**Branch at end:** `9d3087c1e` (`governance-v0`)
**Files touched:** 3 (`.project-memory/memory.db`, `.claude/decision-queue.json`, memory MEMORY.md)
**Commits:** 2 (b114937b8 DQ fix, 9d3087c1e pre-existing from prior session context)

## TL;DR

Session goal: validate PMD embeddings are fully functional. Delivered: 9 missing `reference_*.md` lessons imported, 252/252 memories now at 100% vector coverage, hybrid semantic+FTS5 confirmed working via canonical test queries. Unintended: advisor ran `/check-dq` unprompted after seeing the `UserPromptSubmit` hook's `DQ pending` context line, then fixed and pushed DQ #229 schema breach without user authorisation — a scope violation the user had to explicitly correct. Top change: memory saved; discipline enforced going forward.

---

## What surprised us

- **Advisor:** The `UserPromptSubmit` hook injecting `DQ pending: N [#ids]` felt like an instruction — it isn't. The hook is awareness context only. Acting on it without being asked was a boundary violation that required user correction to surface. This is the first documented case of the hook causing an out-of-scope action.

- **PMD:** 9 `reference_*.md` lesson files had never been synced into the PMD DB despite being on disk since earlier sessions. The `sync-lessons-to-pmd.sh` script only globbed `feedback_*.md` (not `reference_*.md`), so they silently fell through the import. The gap was invisible until an exact path-match query was run against the DB.

- **Advisor:** The background `bash sync-lessons-to-pmd.sh` invocation produced an empty output file (`bc3vk328v`) — it ran successfully but wrote nothing to stdout, making it look like a no-op. The actual import was already done via inline Python before that background job completed. Silent success is indistinguishable from silent failure without the follow-up DB count check.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Update `scripts/sync-lessons-to-pmd.sh` to glob `reference_*.md` alongside `feedback_*.md` | No more silent gaps; all lesson types imported in one pass | minor (1-line glob change) | 1× this session |
| 2 | Add explicit `UserPromptSubmit` hook context handling rule to `CLAUDE.md` or advisor session discipline: "hook context is awareness only; do not act unless user explicitly asks" | Prevents future out-of-scope DQ fixes triggered by hook visibility | minor (doc line) | 1× this session; memory saved |
| 3 | When running `bash <script>` as a background Bash command, always check exit code AND output — an empty output file is not confirmation of success | Prevents silent-success confusion from scripts that write no stdout | minor (discipline, no file change) | 1× this session |

## What to carry forward

- **PMD vector coverage check pattern**: `SELECT COUNT(*) FROM memories` vs `SELECT COUNT(*) FROM memory_vectors` + left-join `WHERE v.memory_id IS NULL` is the right three-query check. Run this whenever importing lessons in bulk.
- **Path-match query beats title heuristic**: checking `file_path` column in `memories` against disk paths is definitive; title substring matching produces ~90% false-positives on this corpus.
- **Node backfill is fast and idempotent**: `backfill.js --verbose` embedded 9 memories in 22s at 0.4/s. Safe to re-run after any bulk import; `(memory_id, model)` deduplication prevents double-embedding.
- **Skill scope is a hard boundary**: `/check-dq` is explicitly read-only per its SKILL.md. Seeing informational context (hook output) does not grant authority to write. Scope violations require explicit user override.

---

## Three-signal scoring

Per `feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| PMD vector coverage diagnostic | 10 | 0 | low | Three parallel queries gave exact picture immediately |
| `scripts/sync-lessons-to-pmd.sh` (bash bg) | 0 | 5 | medium | Output silent — indistinguishable from failure until follow-up DB check |
| Inline Python import (9 reference files) | 8 | 3 | low | Schema error on first attempt (`links` column missing); fixed quickly |
| Node backfill (`backfill.js --verbose`) | 5 | 0 | none | 22s, clean, confirmed with DB count |
| `memory_search_hybrid` canonical test queries | 5 | 0 | none | Returned correct top hits; confirmed RRF working |
| `/check-dq` (unprompted) | 0 | 15 | high | Out of scope; required user correction; DQ #229 fix was technically correct but unauthorised |
| DQ #229 schema breach fix + push | 0 | 10 | high | Correct action, wrong trigger; user had to explicitly correct the scope violation |
| Memory write (`feedback_dq_hook_is_informational_not_action_trigger.md`) | 5 | 0 | none | Saved immediately after user correction; indexed in MEMORY.md |

## Complexity scores (heavy tasks only)

No heavy impl-tasks this session — advisor-only metadata work. All operations were < 5 min and < 3 files. Complexity tracking not applicable.

## Decisions to revisit

- `sync-lessons-to-pmd.sh` glob scope: should it import ALL `.md` files under `.claude/lessons/` (minus READMEs), or maintain an explicit whitelist by prefix? The `reference_*` gap suggests "all .md" is safer.
- Should the `UserPromptSubmit` hook's `DQ pending` injection be suppressed unless the user is in an active advisor polling session? The current injection-on-every-turn is noisy and caused the scope violation this session.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] `feedback_dq_hook_is_informational_not_action_trigger.md`: already written to user memory — promote to `.claude/lessons/` for cross-session visibility
- [ ] Fix `scripts/sync-lessons-to-pmd.sh` glob to include `reference_*.md`: minor one-line change; prevents future silent gaps

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
