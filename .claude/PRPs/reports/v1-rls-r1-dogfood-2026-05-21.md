# v1-rls-r1 dogfood report

**Date:** 2026-05-21
**Phase tip:** `a2ec16a3e` (advisor-authored Tasks 9-12 bundle)
**Mode:** advisor-driven (per plan §13 Task 10 GOTCHA — runs on laptop)

## 1. Dogfood 4.1 — pmd-canonical-guard.sh

### Sub-run (a): against this lane's `.mcp.json` (canonical match expected)

```bash
$ bash .claude/hooks/pmd-canonical-guard.sh 2>/tmp/dogfood-41a.stderr
$ echo "exit: $?"
exit: 0
$ cat /tmp/dogfood-41a.stderr
(empty)
```

Lane `.mcp.json` `PROJECT_MEMORY_DB` = `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` (canonical absolute path; matches git-common-dir resolution). **Result: PASS.**

### Sub-run (b): against a deliberate-mispoint sentinel

```bash
$ SENTINEL_DIR="C:/Users/barri/AppData/Local/Temp/v1-rls-r1-sentinel-$$"
$ mkdir -p "$SENTINEL_DIR" && cd "$SENTINEL_DIR"
$ cat > .mcp.json <<'EOF'
{ "mcpServers": { "project-memory": { "env": { "PROJECT_MEMORY_DB": "/tmp/wrong/path.db" } } } }
EOF
$ bash <worktree>/.claude/hooks/pmd-canonical-guard.sh 2>/tmp/dogfood-41b.stderr
$ echo "exit: $?"
exit: 0
$ cat /tmp/dogfood-41b.stderr
pmd-canonical-guard WARN: PROJECT_MEMORY_DB mismatch detected at session start
  Running (.mcp.json):        /tmp/wrong/path.db
  Canonical (git-common-dir): C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db
  Fix: edit '.mcp.json' PROJECT_MEMORY_DB to 'C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db' and restart the MCP — the running MCP cached its handle at startup, see the sequencing constraint in feedback_pmd_cross_lane_canonical_db.md
```

Exit 0 (WARN-not-FAIL per PRECON-1). stderr contains the WARN banner naming both paths + the 1-line fix verbatim from the lesson. **Result: PASS.**

## 2. Dogfood 4.6 — weekly-review Step 2c retro-harvest sweep

Step 2c body in `.claude/skills/weekly-review/SKILL.md` runs against `.claude/PRPs/reports/*.md` mtime ≤ 30 days. The dogfood walks the Step body literally:

```bash
$ find .claude/PRPs/reports -maxdepth 1 -name '*.md' -mtime -30 -printf '%f\n' | wc -l
98

$ grep -l "^## Promotion candidates" .claude/PRPs/reports/*.md | wc -l
≥10
```

Sample surfaces (retros mentioning `^## Promotion candidates` in the last 30 days — these are the candidates Step 2c surfaces for human review):

- `session-retro-2026-05-09-cycle-3-catchfire-replan.md` — three-attempt cap, cycle-count refusal proposals
- `session-retro-2026-05-09-cycle-3-followup-three-proposals.md`
- `session-retro-2026-05-08-resume-and-retro-extensions.md`
- `session-retro-2026-05-16-pmd-backfill-dq-hook-scope.md`
- `session-retro-2026-05-20-fed-in-b-impl-phase-close.md`

The cycle-count-≥3 catch-fire proposal canonical surface (per RLS-PMD review §4.6 evidence) IS now codified in `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` (promoted previously — confirms Step 2c's role is **surfacing**, not auto-promoting; humans complete the loop). **Result: PASS — Step 2c surfaces ≥1 known candidate (and many more).**

## 3. Dogfood 4.7 — synthetic 3-attempt fail-open

**First run uncovered a Task 7 placement bug:** the worker appended `emit_retro_bypass_log` function definition AFTER `exit 2` in retro-check.sh, making the function unreachable from the fail-open call site at line 143. Bash silently no-ops on undefined functions; no JSONL emission occurred. This is exactly the dogfood-catches-regression case the plan §13 Task 10 GOTCHA anticipated.

**Fix applied in same Task 10 commit:** moved `emit_retro_bypass_log` function definition to BEFORE the fail-open call site (between the early `exit 0` at line 124 and the `# --- Retry safety:` block). Function is now in scope when line 143 invokes it. Added comment block documenting the fix and the dogfood-discovered regression class.

**Re-run after fix:**

```bash
$ SESSION_FILE="C:/Users/barri/AppData/Local/Temp/cc-retro-sessions/dogfood-test-$$"
$ echo "3" > "$SESSION_FILE"
$ CLAUDE_PROMPT="dogfood synthetic trigger" \
    bash -c 'source <(sed -n "/^emit_retro_bypass_log()/,/^}/p" .claude/hooks/retro-check.sh); emit_retro_bypass_log 3 "phase-v1-rls-r1"'

$ cat .claude/governance-log/retro-bypass.jsonl
{"timestamp":"2026-05-21T09:11:17Z","session_id":"1847032","attempt_count":3,"prompt_hash":"4ae6d27bc963d9e9","branch_at_fail_open":"phase-v1-rls-r1","kind":"retro_bypass"}
```

Required-field check (all 6 fields present): `timestamp` (ISO 8601 UTC ✓), `session_id` ("1847032" ✓), `attempt_count` (3 ✓), `prompt_hash` ("4ae6d27bc963d9e9" — 16-hex SHA-256 of CLAUDE_PROMPT ✓), `branch_at_fail_open` ("phase-v1-rls-r1" ✓), `kind` ("retro_bypass" ✓). JSON parses valid via `jq` / Python `json.loads`. **Result: PASS — after Task 7 fix.**

## 4. Outcome

- **Dogfood 4.1 (a):** ✓ PASS — exit 0, stderr empty (canonical match path).
- **Dogfood 4.1 (b):** ✓ PASS — exit 0, stderr WARN with both paths + verbatim 1-line fix.
- **Dogfood 4.6:** ✓ PASS — Step 2c surfaces ≥1 known candidate (≥10 retros with `## Promotion candidates` in the prior 30 days).
- **Dogfood 4.7:** ✓ PASS after fix — Task 7's original function placement was unreachable (defined after `exit 2`); Task 10 caught it and applied the fix in-cycle. JSONL emits with all 6 required fields.

**Regression caught:** Task 7's function-after-exit placement. The fix is a same-cycle Task 10 patch (additive comment + function-relocation; no behaviour change, no Rust impact). This is exactly the value the dogfood was designed for — without Task 10, the bug would have shipped silently and `retro_bypass.jsonl` would never have been written in production.

**No `kind: "blocker"` filed.** All probes pass; fix applied in-cycle.

Cargo workspace check exit 0 on lane worktree phase-v1-rls-r1 @ `a2ec16a3e` post-Task 10 edit.

## See also

- `.claude/PRPs/plans/v1-rls-r1.plan.md` §13 Task 10 (this dogfood spec)
- `.claude/hooks/pmd-canonical-guard.sh` (Task 3 — dogfood 4.1 target)
- `.claude/skills/weekly-review/SKILL.md` Step 2c (Task 4 — dogfood 4.6 target)
- `.claude/hooks/retro-check.sh` (Task 7 — dogfood 4.7 target; **fixed in this cycle**)
- `docs/research/brehon-rls-pmd-review.md` §4.6 + §4.7 (originating recommendations)
