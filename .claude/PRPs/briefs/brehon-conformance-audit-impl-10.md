# brehon-conformance-audit — impl Task 10 brief (Cohort 2)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit Task 10 — rust-analyzer-mcp install + .mcp.json.example — see .claude/PRPs/briefs/brehon-conformance-audit-impl-10.md`

## 2. Scope

Implement plan §13 **Task 10** (lines 1357-1417) of `.claude/PRPs/plans/brehon-conformance-audit.plan.md` (revised at `7bb6c51bb`; phase tip `e3b62793f`). One commit, one file modified:

```yaml
creates: []
modifies:
  - .mcp.json.example
requires:
  - task: 0
    reason: "Task 0 verifies environment baseline; install is a side-effect, not a code change."
```

Install `rust-analyzer-mcp` via `cargo install rust-analyzer-mcp` (idempotent — skip if already installed). Update `.mcp.json.example` with `rust-analyzer` server entry per §10.10 verbatim. Document `rustup component add rust-analyzer` prereq in `_comment` field. Do NOT touch any per-lane `.mcp.json` (gitignored).

**Do NOT** author:
- SKILL.md, axis sub-files, find-sibling.sh, audit-metrics.schema.json, METRICS.md, clippy.toml (Tasks 1, 2, 3, 4, 5, 8 peers/spine).
- compute-metrics.sh (Task 6, Cohort 2.5).
- Per-module deny attributes (Task 9).
- Any per-lane `.mcp.json` (each lane bootstraps its own from `.mcp.json.example`).
- Anything under `crates/`, `migrations/`, `tests/`.

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (at `e3b62793f`).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on worker branch; daemon finalize-merges.

## 3. Required reading

### 3.0 Plan + PRECON sources (P0)

1. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.10 (rust-analyzer-mcp `.mcp.json.example` entry — verbatim), §13 Task 10 (lines 1357-1417).
2. `.mcp.json.example` (existing — repo root) — Read first to see merge target; preserve all existing entries.

### 3.1 MIRROR — existing .mcp.json.example entries

3. `.mcp.json.example` server-entry shape (server name → command + args + env). Mirror that shape.

### 3.2 Multi-lane discipline

4. `.claude/rules/multi-lane-worktree.md` §"PMD is cross-lane shared, NOT per-lane isolated" — the rust-analyzer-mcp install is to the laptop's user-scope cargo bin; the .mcp.json.example template carries the absolute path so per-lane bootstraps inherit it (analogous to how `PROJECT_MEMORY_DB` is canonical).
5. `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` — companion lesson.

### 3.3 Rules (auto-loaded)

6. `.claude/rules/decision-queue.md` — mid-task visibility; cargo-install failure blocker shape.
7. `.claude/rules/phase-branch.md` — worker branch push.

## 4. Constraints

1. **Mid-task push discipline** — any DQ pushed immediately to worker branch.
2. **Attribution integrity** — `from: "impl"`; never `answered_by: "advisor"`/`"user"`.
3. **`cargo install rust-analyzer-mcp` is IDEMPOTENT** — re-running on already-installed crate is a no-op. Worker MAY skip the install if `which rust-analyzer-mcp` exits 0.
4. **DO NOT touch any per-lane `.mcp.json`** — those are gitignored bootstrap targets. Only `.mcp.json.example` (committed template) is modified.
5. **Preserve all existing `.mcp.json.example` entries** — merge `rust-analyzer` server entry, do NOT replace the file.
6. **`_comment` field documents `rustup component add rust-analyzer` prereq** verbatim per §10.10.
7. **NO writes to `.claude/skills/**`** in this task (skill files are Tasks 1, 2 peers).
8. **Workspace cargo-check is required** — run bash scripts/brehon/cargo-check.sh --workspace --features full per section 5.3 before push.
9. **On cargo-install failure** — file `kind: "blocker"` DQ to advisor; do NOT commit anything. Failed install is a prereq for Story 3 verification per plan §13 Task 10 GOTCHA (but does NOT block dogfood — Task 3 find-sibling.sh has Grep+Read fallback).
10. **Commit subject template** — `feat(mcp): add rust-analyzer-mcp to .mcp.json.example (task 10)`.
11. **Single commit** per plan §13 norm. Lockfile is NOT modified by this task (no Cargo.toml changes; cargo install does not touch Cargo.lock).
12. **No `--no-verify`** — never skip hooks.

## 5. Validation gate (DoD per plan §15)

### 5.1 Install probe (worker runs locally before push)

```bash
# Install (idempotent — cargo install is no-op if installed)
cargo install rust-analyzer-mcp 2>&1 | tee .claude/PRPs/debug/brehon-conformance-audit-task10-install.log
echo "exit: $?"
# EXPECT: exit 0

# Verify rust-analyzer (the LSP server itself) is on PATH
rustup which rust-analyzer 2>/dev/null || { echo "RUST_ANALYZER MISSING — run: rustup component add rust-analyzer"; exit 1; }
echo "exit: $?"
# EXPECT: exit 0

# Verify rust-analyzer-mcp callable
which rust-analyzer-mcp
echo "exit: $?"
# EXPECT: exit 0; stdout is the install path
```

### 5.2 .mcp.json.example structural check

```bash
# rust-analyzer entry present
grep -F "rust-analyzer" .mcp.json.example
# EXPECT: at least one match

# .mcp.json.example still valid JSON
python3 -c "import json; json.load(open('.mcp.json.example')); print('OK')"
# EXPECT: OK
```

### 5.3 Cargo-check (workspace sanity — no Cargo.toml change so should pass)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-task10-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/brehon-conformance-audit-task10-check.log
# EXPECT: exit 0
```

### 5.4 §15 validate-pending-laptop DQ (raise after worker push)

```json
{
  "id": "<next>",
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO>",
  "question": "validate phase-brehon-conformance-audit task 10 worker branch",
  "branch": "<worker-branch>",
  "phase_task": 10,
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-task10-check.log 2>&1; echo exit: $?"
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

DQ entry committed + pushed BEFORE worker exits.

## 6. KNOWN harness limitations

1. **CC v2.1.119 sensitive-file gate** — `.mcp.json.example` at repo root is NOT under `.claude/`; gate should NOT fire. If it does, `/tmp + mv` fallback.
2. **Daemon finalize-merge no-push** — advisor handles SSH-push.
3. **EliteDesk daemon does NOT need rust-analyzer-mcp** — cargo runs on laptop per PRECON-2; Junior workers don't invoke LSP via MCP (LSP is an agent built-in tool, not an MCP server). The .mcp.json.example update is for laptop sessions' per-lane bootstraps.
4. **`cargo install` may need network** — if EliteDesk daemon worker has no network, cargo-install fails; worker files blocker DQ.

## 7. Next steps after this task

- Daemon finalize-merges Task 10 commit onto phase branch.
- Advisor polling + validate-pending-laptop mutation.
- After ALL Cohort 2 tasks pass, dispatch Cohort 2.5 (Task 6 — compute-metrics.sh).
- Task 7 (Cohort 3 dogfood) uses rust-analyzer-mcp + find-sibling.sh's LSP path.
- Task 12 (Cohort 5 lessons) records the lane-rebootstrap UX cost.

## 8. Commit subject template (verbatim)

```
feat(mcp): add rust-analyzer-mcp to .mcp.json.example (task 10)
```

Body cites plan path + SHA + cargo-install output + verify-on-PATH evidence.

## 9. DoD for this brief (advisor-side gate — completed at brief commit)

- [x] Brief committed on `phase-brehon-conformance-audit` BEFORE Junior task dispatch.
- [x] Cites plan §13 Task 10 line range (1357-1417).
- [x] §4 enumerates 12 constraints.
- [x] §5 names validate-pending-laptop DQ + install-failure blocker shape.
