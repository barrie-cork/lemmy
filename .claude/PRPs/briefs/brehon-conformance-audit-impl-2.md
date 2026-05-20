# brehon-conformance-audit — impl Task 2 brief (Cohort 2)

## 1. Role + dispatch line

`[role:impl-task] brehon-conformance-audit Task 2 — six axis sub-files — see .claude/PRPs/briefs/brehon-conformance-audit-impl-2.md`

## 2. Scope

Implement plan §13 **Task 2** (lines 837-937) of `.claude/PRPs/plans/brehon-conformance-audit.plan.md` (revised at `7bb6c51bb`; phase tip `e3b62793f`). One commit, six files created:

```yaml
creates:
  - .claude/skills/brehon-conformance-audit/axes/1-conn-type.md
  - .claude/skills/brehon-conformance-audit/axes/2-append-reborrow.md
  - .claude/skills/brehon-conformance-audit/axes/3-trait-bound.md
  - .claude/skills/brehon-conformance-audit/axes/4-error-idiom.md
  - .claude/skills/brehon-conformance-audit/axes/5-conn-acquisition.md
  - .claude/skills/brehon-conformance-audit/axes/6-adr-015.md
modifies: []
requires:
  - task: 1
    reason: "Each axis sub-file inherits allowed-tools from SKILL.md and references the parent skill name."
```

Axes #1, #3, #4 are **judgment-heavy** (populated Error-code → design-question tables). Axes #2, #5, #6 are **byte-conformance** (no table; simple grep specs). Six files in ONE commit per plan §13 norm.

**Do NOT** author:
- SKILL.md skeleton (shipped Task 1 @ `1e3948e43`).
- Scripts (`find-sibling.sh`, `compute-metrics.sh` — Tasks 3, 6).
- Metrics schema or METRICS.md (Tasks 4, 5).
- `clippy.toml` (Task 8).
- Per-module deny attributes in federation `mod.rs` (Task 9).
- `.mcp.json.example` edit (Task 10).
- Anything under `crates/`, `migrations/`, `tests/`, `docs/brehon-law-inspired-network/`.

**Branch context:**
- `base_branch` = `phase-brehon-conformance-audit` (at `e3b62793f`).
- Worker forks to its own `junior/...` worktree branch.
- ONE commit on the worker branch; daemon finalize-merges into `phase-brehon-conformance-audit`.

## 3. Required reading

### 3.0 Plan + governing brief (P0)

1. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` §10.3 (axis sub-file structure — verbatim), §13 Task 2 (lines 837-937 — six IMPLEMENT blocks).
2. `.claude/skills/brehon-conformance-audit/SKILL.md` (Task 1 output) — the spine these axes attach to; read first to confirm forward-reference paths align.

### 3.1 PMD + ADR sources for axis content

3. PMD `project_phase6_convention_divergence_class.md` — the six-axis schema source; user-directive memory. Lift verbatim into each axis sub-file's Trace Up ↑.
4. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — ADR-013 (`CaseStatus::EmergencyRemove` — adjacent to axis #4) + ADR-015 (`actor_pseudonym` — axis #6 source).

### 3.2 Lessons (mandatory per §2.4 file-class table — none of axes/*.md match the table, but content sources do)

5. `feedback_multi_write_handlers_need_transactions.md` — axes #1, #2, #6 Trace Up source.
6. `feedback_async_pool_test_pattern.md` — axes #1, #5 Trace Up source.
7. `feedback_lemmy_error_no_std_error.md` — axes #3, #4 Trace Up source; Case A/B/C enumeration.
8. `feedback_clippy_test_style.md` — axis #4 cross-reference to Track B.

### 3.3 Rules (auto-loaded)

9. `.claude/rules/decision-queue.md` — mid-task visibility; attribution integrity.
10. `.claude/rules/phase-branch.md` — worker branches push to origin via daemon-finalize-merge.
11. `.claude/rules/pmd-search-strategy.md` — hybrid search precedence.

### 3.4 Canonical MIRROR

12. `.claude/skills/brehon-conformance-audit/SKILL.md` § structure references the per-axis sub-file structure. The axis sub-files follow plan §10.3 verbatim — Read it first.

## 4. Constraints

1. **Mid-task push discipline** per `decision-queue.md`: any DQ entry MUST be committed AND pushed to the worker branch IMMEDIATELY (`git push origin <worker-branch>`) before blocking on an answer.
2. **Attribution integrity**: `from: "impl"`. NEVER `answered_by: "advisor"` or `"user"`. Self-resolve with `answered_by: "impl-self-resolved"` ONLY when answer is in codebase / plan / lessons.
3. **No cargo from skill body** — axis sub-files are documentation only; do NOT embed cargo commands (Watchpoint #3).
4. **Frontmatter `allowed-tools` exactly `[LSP, Read, Grep, Glob, Bash]`** on every axis sub-file (PRECON-3, inherited from SKILL.md).
5. **Six files, NOT five, NOT seven** — exactly the six paths in §2 above. Per `feedback_impl_task_enumerated_transform_all_or_blocker.md`: enumerated all-or-blocker.
6. **Axes #2, #5, #6 are byte-conformance — DO NOT invent Error-code table content** for them (plan §10.3 GOTCHA explicit).
7. **Axes #1, #3, #4 carry populated Error-code → design-question tables** sourced from cited lessons + Finding 6.1 + fix-impl-1 traces. Use real error codes (E0599, E0277, etc) — no inventions.
8. **Evidence string format** EXACTLY as per §10.3 / per-axis IMPLEMENT block. The compute-metrics.sh script (Task 6) parses these strings; deviation breaks downstream metrics.
9. **Cross-reference Track B in axis #4** verbatim per plan §13 Task 2 IMPLEMENT (file 4 of 6).
10. **Commit subject template** — `feat(skill): add brehon-conformance-audit six axis sub-files (task 2)`. Body cites plan + commit + which axes were judgment vs byte-conformance.
11. **Single commit** per plan §13 norm.
12. **No `--no-verify`** — never skip hooks.

## 5. Validation gate (DoD per plan §15)

### 5.1 Pre-push structural check (worker runs locally before push)

```bash
# Per plan §13 Task 2 VALIDATE (lines 912-936):
for f in .claude/skills/brehon-conformance-audit/axes/*.md; do
  echo "=== $f ==="
  head -1 "$f"
  grep -c "^## " "$f"
done

# Frontmatter parse:
python3 -c "
import yaml, re, glob
files = sorted(glob.glob('.claude/skills/brehon-conformance-audit/axes/*.md'))
assert len(files) == 6, f'expected 6, got {len(files)}'
for f in files:
    content = open(f).read()
    m = re.match(r'^---\n(.*?)\n---', content, re.S)
    assert m, f'no frontmatter in {f}'
    fm = yaml.safe_load(m.group(1))
    assert 'axis' in fm and 'title' in fm, fm
    assert fm['allowed-tools'] == ['LSP', 'Read', 'Grep', 'Glob', 'Bash'], fm
print('OK')
"
# EXPECT: OK
```

### 5.2 Cargo-check (workspace clean)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-task2-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/brehon-conformance-audit-task2-check.log
# EXPECT: exit 0
```

### 5.3 §15 validate-pending-laptop DQ (raise after worker push)

Per advisor-orchestrator §5.2: after the worker pushes the worker branch, raise:

```json
{
  "id": "<next>",
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO>",
  "question": "validate phase-brehon-conformance-audit task 2 worker branch",
  "branch": "<worker-branch>",
  "phase_task": 2,
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/brehon-conformance-audit-task2-check.log 2>&1; echo exit: $?"
  ],
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

The DQ entry MUST be committed + pushed to the worker branch BEFORE the worker exits.

## 6. KNOWN harness limitations (read + plan around — do NOT try to bypass)

1. **CC v2.1.119 sensitive-file gate on `.claude/skills/**`** — `.mcp.json` allow-list covers this path, but the gate may still fire interactively. If a Write into `.claude/skills/brehon-conformance-audit/axes/*.md` is blocked, fall back to the `/tmp + mv` workaround per plan §19.5 LESSON (write to `/tmp/<file>.md` then `mv /tmp/<file>.md .claude/skills/brehon-conformance-audit/axes/<n>-<title>.md`).
2. **Edit tool gate on plan files** — settings.json was broadened 2026-05-20 for `.claude/PRPs/plans/**`. Axes are under `.claude/skills/**` which was already allowed; the Edit gate should NOT fire on this path class, but `/tmp + mv` is the fallback.
3. **Daemon finalize-merge no-push** — known pattern; advisor handles SSH-push post-finalize. NO worker action needed.

## 7. Next steps after this task

- Daemon finalize-merges Task 2 commit onto `phase-brehon-conformance-audit`.
- Advisor polling loop detects new tip + `validate-pending-laptop` DQ entry.
- Advisor laptop runs cargo-check + mutates DQ to `result: "pass"` (or §G4 classifies).
- After ALL Cohort 2 tasks (2, 3, 4, 5, 8, 10) reach `result: "pass"`, advisor dispatches Cohort 2.5 (Task 6 alone).

## 8. Commit subject template (verbatim)

```
feat(skill): add brehon-conformance-audit six axis sub-files (task 2)
```

Commit body MUST cite:
- Plan path + SHA: `.claude/PRPs/plans/brehon-conformance-audit.plan.md (e3b62793f)`.
- §13 Task 2 IMPLEMENT (file 1..6) authored.
- Axes #1/#3/#4 judgment-heavy (populated tables); axes #2/#5/#6 byte-conformance.
- Co-Authored-By trailer.

## 9. DoD for this brief (advisor-side gate — completed at brief commit)

- [x] Brief committed on `phase-brehon-conformance-audit` BEFORE Junior task dispatch.
- [x] Brief cites plan §13 Task 2 line range (837-937).
- [x] Brief lists ≤ 4 P0 reads + axis-content sources.
- [x] Brief §4 enumerates 12 hard constraints.
- [x] Brief §5 names the validate-pending-laptop DQ shape verbatim.
- [x] Brief §6 carries KNOWN harness limitations.
