# Plan — Mirror 5 missing infra lessons into repo `.claude/lessons/`

**Status:** ready to execute (new session)
**Author:** advisor (retro-harvest Tier-1 finding, 2026-05-16)
**Branch:** `governance-v0` (direct-commit meta-work per `.claude/rules/phase-branch.md` — these are `.claude/lessons/` files, NOT code; no phase branch, no PR)
**Type:** mirror + frontmatter-transform + PMD re-index. NOT authoring — the content already exists.

---

## 1. Problem (verified ground truth, 2026-05-16 @ HEAD `50ec13495`+)

The retro-harvest skill surfaced a Tier-1 data-quality gap. Five lessons are:

- **Referenced as live/authoritative** in `MEMORY.md` (the always-loaded user-memory index) at lines 63, 66, 71, 139, 173.
- **Cited by retros** (SL-lane-meta §5, JM-d §5, v1-validate-agent, context-injection-optimisation, pmd-backfill-dq-hook-scope #2) as "promoted to PMD — mirror to `.claude/lessons/` pending".
- **Absent from the repo**: `ls .claude/lessons/<name>.md` → no match for all 5; `git log --all --diff-filter=A` → 0 add-commits for all 5. Never committed under any name.
- **Present, substantive, in user-memory**: `C:/Users/barri/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/<name>.md` — 69 / 14 / 77 / 15 / 29 lines respectively. The two short ones (14, 15) are complete, not stubs.

**Impact:** Junior subagents `Glob .claude/lessons/feedback_*.md` at session start and `sync-lessons-to-pmd.sh` imports that dir into the brehon-fork PMD. Because these 5 live only in user-memory (laptop-local, not repo-tracked, not synced), **the cross-session Junior visibility the retros assumed does not exist** — and `MEMORY.md` is a dangling index pointing at repo paths that aren't there. One of them (`feedback_daemon_local_trunk_stale_multi_lane.md`) is cited by a *mandatory* `~/.claude/commands/precheck.md` step, so the most load-bearing multi-lane safety reference points at nothing.

This is exactly the silent-drop class the retro-harvest skill exists to catch, applied to itself.

## 2. The 5 lessons (source → destination)

| # | Lesson name | Source (user-memory, 204 lines total) | Dest (repo) | MEMORY.md line |
|---|---|---|---|---|
| 1 | `feedback_daemon_local_trunk_stale_multi_lane` | `…/memory/feedback_daemon_local_trunk_stale_multi_lane.md` (69 ln) | `.claude/lessons/feedback_daemon_local_trunk_stale_multi_lane.md` | 63 |
| 2 | `feedback_dq_hook_is_informational_not_action_trigger` | `…/memory/feedback_dq_hook_is_informational_not_action_trigger.md` (14 ln) | `.claude/lessons/feedback_dq_hook_is_informational_not_action_trigger.md` | 66 |
| 3 | `feedback_verify_automated_reviewer_claims_against_compiler` | `…/memory/feedback_verify_automated_reviewer_claims_against_compiler.md` (77 ln) | `.claude/lessons/feedback_verify_automated_reviewer_claims_against_compiler.md` | 71 |
| 4 | `feedback_claude_md_invariants_vs_state` | `…/memory/feedback_claude_md_invariants_vs_state.md` (15 ln) | `.claude/lessons/feedback_claude_md_invariants_vs_state.md` | 173 |
| 5 | `feedback_lane_dq_resolution_append_to_trunk` | `…/memory/feedback_lane_dq_resolution_append_to_trunk.md` (29 ln) | `.claude/lessons/feedback_lane_dq_resolution_append_to_trunk.md` | 139 |

`…/memory/` = `C:/Users/barri/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/`

## 3. The frontmatter transform (REQUIRED — not a raw copy)

User-memory and repo-lesson frontmatter shapes differ. `sync-lessons-to-pmd.sh` parses `name`, `description`, `type` from flat top-level keys (see the script's `parse_lesson()` — it reads `fm.get("name")`, `fm.get("type")` from a flat dict; a nested `metadata:` block makes `type` parse as empty). So each file MUST be transformed, not copied.

**Source shape (user-memory):**
```yaml
---
name: daemon-local-trunk-stale-multi-lane
description: "…"
metadata:
  node_type: memory
  type: feedback
  originSessionId: dcdeac73-…
---
```

**Destination shape (repo-lesson canonical — match `feedback_runbook_audit_drift_post_event_check.md`):**
```yaml
---
name: <Title Case human-readable name — see step note>
description: <same description text, drop the surrounding quotes if present>
type: feedback
---
```

Transform rules:
- `name:` → repo-lessons use a human-readable Title Case sentence, not the kebab slug. Use the MEMORY.md link-text as the name (e.g. line 63 `[Daemon-local trunk stale in multi-lane]` → `name: Daemon-local trunk stale in multi-lane`). It is descriptive and already the canonical handle.
- `description:` → keep verbatim; strip wrapping `"`/`'` if the source quoted it.
- `metadata:` block → collapse to a single flat `type: feedback` line. Drop `node_type`, `originSessionId` (those are user-memory-system fields, meaningless in the repo lesson corpus).
- **Body** → copy verbatim, byte-for-byte. Do not re-summarise, re-order, or "improve" — these are validated lessons; the body is the contract.

## 4. Watchpoints (specific, per `feedback_advisor_watchpoint_specificity`)

- **W1 — `sync-lessons-to-pmd.sh` glob.** The script's `for lesson in "$LESSONS_DIR"/feedback_*.md "$LESSONS_DIR"/reference_*.md` loop (verified present at HEAD after commit `50ec13495`) globs both `feedback_*` and `reference_*`. All 5 dest names are `feedback_*` → in scope. No script change needed. Confirm the glob line still reads both prefixes before running step 5.4 (regression guard — the `reference_*` arm was only added 2026-05-16).
- **W2 — UTF-8 on Windows.** `feedback_verify_automated_reviewer_claims_against_compiler.md` (77 ln) and the daemon one contain `→`, `≥`, backticked code. Per `feedback_python_utf8_encoding_windows`: any Python round-trip MUST pass `encoding="utf-8"`. Prefer the `Read` tool + `Write` tool (UTF-8-safe) over a Python copy script. If using `cp` via Bash, that's byte-safe (no re-encode) — but the frontmatter transform still needs an editor, so Read+Write per file is the cleanest path.
- **W3 — `sync-lessons-to-pmd.sh` idempotency key is the `name:` field.** The script skips a lesson if `SELECT COUNT(*) FROM memories WHERE title = '<name>'` > 0. If a *different* title was already imported for related content, the new file silently won't import. After step 5.4, explicitly verify all 5 new titles landed (step 5.5) — do not trust the script's exit code alone (per `pattern_verify_before_trusting_shell_output`).
- **W4 — backfill embeds only rows lacking vectors.** `sync-lessons-to-pmd.sh` inserts rows WITHOUT embeddings (it's a raw SQLite insert, bypasses the MCP embed path). The node `backfill.js` must run after, or the 5 lessons are FTS5-only (semantic search misses them). This is the exact gap from the 2026-05-16 PMD session — do not skip step 5.6.
- **W5 — MEMORY.md lines already exist and are correct.** Lines 63/66/71/139/173 already point at the right repo paths. Once the files exist, the index becomes valid automatically — **do NOT edit MEMORY.md**. Editing it is out of scope and risks the cp1252 corruption class (`feedback_python_utf8_encoding_windows`). The dangling index self-heals when the files land.

## 5. Tasks (execute in order; one commit at the end)

### 5.1 Pre-flight (verify ground truth still holds)
```
cd C:/Users/barri/Developer/brehon-fork
git fetch origin governance-v0 && git status --short          # clean tree expected; if dirty, STOP & surface
git rev-parse --short HEAD                                      # record for the commit body
for f in feedback_daemon_local_trunk_stale_multi_lane feedback_dq_hook_is_informational_not_action_trigger feedback_verify_automated_reviewer_claims_against_compiler feedback_claude_md_invariants_vs_state feedback_lane_dq_resolution_append_to_trunk; do test -f ".claude/lessons/$f.md" && echo "UNEXPECTED: $f.md already exists — STOP, re-triage" || echo "ok-missing $f"; done
```
If any file unexpectedly already exists, STOP — a concurrent session may have done this; re-run retro-harvest before proceeding.

### 5.2 For each of the 5 lessons (Read source → transform frontmatter → Write dest)
For lesson N (1..5 per the table in §2):
1. `Read` the user-memory source file in full.
2. `Read` the corresponding `MEMORY.md` line (63/66/71/139/173) to get the canonical human-readable name for `name:`.
3. `Write` `.claude/lessons/<dest-name>.md` with:
   - Transformed frontmatter per §3 (flat `name`/`description`/`type: feedback`).
   - Body copied verbatim from source (everything after the source's closing `---`).
4. Do not proceed to the next lesson until the current `Write` succeeds.

### 5.3 Sanity-check the 5 written files
```
for f in <5 dest names>; do echo "=== $f ==="; head -5 ".claude/lessons/$f.md"; done
# Expect: each starts with ---, flat name:, description:, type: feedback, --- . No `metadata:` block. No `node_type`.
grep -L "^type: feedback$" .claude/lessons/feedback_daemon_local_trunk_stale_multi_lane.md .claude/lessons/feedback_dq_hook_is_informational_not_action_trigger.md .claude/lessons/feedback_verify_automated_reviewer_claims_against_compiler.md .claude/lessons/feedback_claude_md_invariants_vs_state.md .claude/lessons/feedback_lane_dq_resolution_append_to_trunk.md
# grep -L prints files NOT matching — expect EMPTY output (all 5 have the flat type line)
```

### 5.4 Run the PMD sync (imports the 5 new files as `pattern` rows)
```
# W1 regression guard first:
grep -n 'for lesson in' scripts/sync-lessons-to-pmd.sh   # confirm line globs feedback_*.md AND reference_*.md
bash scripts/sync-lessons-to-pmd.sh --verbose 2>&1 | tee C:/Users/barri/AppData/Local/Temp/sync-lessons-run.log
# Expect: "import" lines for all 5 new titles, "skip (exists)" for the ~114 already present.
```

### 5.5 Verify the 5 landed in PMD (do NOT trust exit code — W3)
```
python3 -c "
import sqlite3
con=sqlite3.connect('.project-memory/memory.db'); cur=con.cursor()
for t in ['Daemon-local trunk stale in multi-lane','DQ hook is informational, not action trigger','Verify automated-reviewer claims against the compiler','CLAUDE.md invariants vs state','Lane DQ resolution → append to trunk']:
    cur.execute('SELECT COUNT(*) FROM memories WHERE title=?',(t,)); print(cur.fetchone()[0], t)
con.close()"
# adjust the title strings to whatever §5.2 step 2 actually used as name: — expect 1 for each
```
If any returns 0, the `name:` in the file ≠ the title queried, OR W3 collision. Inspect, fix the file's `name:`, delete the mis-imported row if any, re-run 5.4.

### 5.6 Backfill embeddings (W4 — semantic search depends on this)
```
OLLAMA_URL=http://homeserver:11434 PROJECT_MEMORY_DB=.project-memory/memory.db PROJECT_ROOT=C:/Users/barri/Developer/brehon-fork node C:/Users/barri/Developer/MCPs/project-memory-mcp/dist/scripts/backfill.js --verbose 2>&1 | tee C:/Users/barri/AppData/Local/Temp/backfill-run.log
# Expect: "N rows to embed" where N == 5 (only the new rows lack vectors), "done=5 failed=0"
python3 -c "
import sqlite3; con=sqlite3.connect('.project-memory/memory.db'); cur=con.cursor()
cur.execute('SELECT COUNT(*) FROM memories'); t=cur.fetchone()[0]
cur.execute('SELECT COUNT(*) FROM memory_vectors'); v=cur.fetchone()[0]
print(f'total={t} vectors={v} missing={t-v}'); con.close()"
# Expect missing=0
```

### 5.7 Functional spot-check (semantic search now finds them)
Use `memory_search_hybrid` (via MCP) for two of the five:
- `query: "Junior worker branches from daemon local trunk not origin stale"` → expect `feedback_daemon_local_trunk_stale_multi_lane` in top 3.
- `query: "verify automated reviewer trait claims against compiler before triage"` → expect `feedback_verify_automated_reviewer_claims_against_compiler` in top 3.
If FTS5-only (Ollama down), note it; the import + file existence is still the load-bearing fix (Junior Glob works regardless of embeddings).

### 5.8 Commit (one commit, direct to governance-v0)
```
git add .claude/lessons/feedback_daemon_local_trunk_stale_multi_lane.md \
        .claude/lessons/feedback_dq_hook_is_informational_not_action_trigger.md \
        .claude/lessons/feedback_verify_automated_reviewer_claims_against_compiler.md \
        .claude/lessons/feedback_claude_md_invariants_vs_state.md \
        .claude/lessons/feedback_lane_dq_resolution_append_to_trunk.md
# Note: .project-memory/memory.db is gitignored — do NOT stage it. Verify with: git status --short
git commit -m "chore(lessons): mirror 5 infra lessons from user-memory into repo corpus

retro-harvest Tier-1 fix: these 5 were referenced live in MEMORY.md
(lines 63/66/71/139/173) + cited by SL-lane-meta/JM-d/validate-agent
retros as 'mirror pending' but were never committed to .claude/lessons/.
Junior subagents Glob that dir + sync-lessons-to-pmd.sh imports it, so
the cross-session visibility the retros assumed did not exist. Content
copied verbatim from user-memory; frontmatter transformed to repo flat
shape (name/description/type) so the PMD importer parses it. MEMORY.md
unchanged — its index lines self-heal now the files exist.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
git push origin governance-v0
```

## 6. Acceptance criteria (all must hold)

1. 5 files exist at `.claude/lessons/feedback_*.md` with flat `type: feedback` frontmatter (no `metadata:` block).
2. Each body is byte-identical to its user-memory source (verbatim copy).
3. `sync-lessons-to-pmd.sh` imported all 5 (verified by title query, not exit code).
4. `memory_vectors` count == `memories` count (missing=0 after backfill).
5. `git status --short` clean except the one commit; `.project-memory/memory.db` NOT staged (gitignored).
6. `MEMORY.md` unmodified (lines 63/66/71/139/173 already correct; they self-heal).
7. One commit on `governance-v0`, pushed.

## 7. Out of scope (do NOT do these)

- **Tier-2 items** (migrate-roundtrip.sh, SL-e test-technique lessons, cr-4 NoAction assertion) — separate, scheduled work; not this plan.
- **Tier-3 box-ticking** (~20 STALE boxes in source retros) — the retro-harvest skill's own hard-refusal: do not mutate source retros. Separate optional cleanup the user runs if desired.
- **The 2 OTHER missing lessons** flagged by some eval runs (`feedback_dq_collision_across_refs.md`, `feedback_planner_clippy_dryrun_implement_bodies.md`) — those are NOT in user-memory (verified: not in the §2 ground-truth check). They'd be genuine *authoring* work from retro bodies, a different task class. If wanted, that's a separate plan. THIS plan is strictly the 5 mirror-only lessons that already have written content.
- **Editing MEMORY.md** — W5: index self-heals; editing risks cp1252 corruption.
- **`advisor-orchestrator.md` §G4 `clippy::as_conversions` row** (retro-harvest Tier-1 #3) — separate rule edit, not a lesson mirror.

## 8. Why this is safe to run in a fresh session

- Self-contained: §2 table has exact source + dest paths; §3 has the precise transform; §5 is copy-pasteable.
- Read-only until §5.8: every step before the commit is Read/Write to new files + read-only DB queries. No existing file mutated (W5).
- Reversible: if anything is wrong, `git reset HEAD~1` + delete the 5 new files + `sqlite3 … DELETE FROM memories WHERE title IN (…)`. No destructive op on existing state.
- Ground truth re-verified at §5.1 — if the world changed (concurrent session already did it), the plan STOPS rather than double-applying.

---

_Source: retro-harvest report `.claude/PRPs/reports/retro-harvest-2026-05-16.md` Tier-1 #1. Plan authored by advisor 2026-05-16; execute in a new session on `governance-v0`._
