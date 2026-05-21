# v1-rls-r1 — impl Task 9 brief (serial chain)

## 1. Role + dispatch line

`[role:impl-task] v1-rls-r1 Task 9 — paired retro_bypass lesson — see .claude/PRPs/briefs/v1-rls-r1-impl-9.md`

## 2. Scope

Implement plan §13 **Task 9** (lines 2021-2124) of `.claude/PRPs/plans/v1-rls-r1.plan.md` (committed `97af1b439`). One commit, one file created:

```yaml
creates:
  - .claude/lessons/feedback_retro_bypass_governance_log.md
modifies: []
requires:
  - task: 7
    reason: "The lesson cites Task 7's emit_retro_bypass_log + the JSONL fields."
  - task: 8
    reason: "The lesson cites Task 8's governance-log-kinds-jsonl.md doc + the retro_bypass kind registry entry."
```

Paired Track D lesson cross-linking the Track C instrumentation (Tasks 7 + 8).

**Do NOT** author:
- Any other artifact (Tasks 10, 11, 12).
- A proposal to change the 3-attempt cap — fix is the TRAIL, not the cap (per Watchpoint #7 + plan §13 Task 9 GOTCHA).
- Cross-links to lesson files that do not exist at HEAD (skip silently per plan §13 Task 9 GOTCHA).
- Any edit under `crates/`, `migrations/`, `tests/`, `docs/brehon-law-inspired-network/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`.

**Branch context:**
- `base_branch` = `phase-v1-rls-r1` (at `97af1b439`).
- Worker forks; one commit; daemon finalize-merges.

## 3. Required reading

### 3.0 Plan + governing brief (P0)

1. `.claude/PRPs/plans/v1-rls-r1.plan.md` §13 Task 9 (lines 2021-2124 — IMPLEMENT spec, MIRROR, GOTCHAs, VALIDATE).
2. `.claude/PRPs/briefs/v1-rls-r1-planning-1.md` §0.1 PRECON-3 (no keyword-stuffing in `description:` field) — BINDING.

### 3.1 Canonical sibling MIRROR (P0 — read BEFORE first Write)

3. `.claude/lessons/feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` — the prior lesson-pairs-with-spec pattern; structural mirror.
4. `.claude/hooks/retro-check.sh` (current state at `97af1b439`) — verify emit_retro_bypass_log is present + reference line numbers in lesson body.
5. `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md` (current state) — confirm `retro_bypass` row in the kind registry table.

### 3.2 Cross-cohort DQ id collision context (P0)

6. `.claude/lessons/feedback_cohort_dq_id_collision.md` — your next_id is **316 or higher** (max id at phase tip `97af1b439` is 315 in pending → 316 next; but recall Task 6/8 workers consumed extra ids via PMD-fallback retro logs, max might already be higher — compute fresh from the phase tip's DQ pending+resolved).

### 3.3 Rules (auto-loaded)

7. `.claude/rules/decision-queue.md` — schema v2; impl-task writes `from: "impl"`.
8. `.claude/rules/phase-branch.md` — worker → daemon-finalize → phase branch.
9. `.claude/rules/branch-manager.md` — file ownership.

## 4. Constraints

1. **YAML frontmatter** verbatim from plan §13 Task 9 IMPLEMENT step 1 (name, description, type=feedback). Plain natural language; NO "DO use when", "MUST USE", "ALWAYS INVOKE", "CRITICAL" verbiage.
2. **Body section order** matches plan §13 Task 9 IMPLEMENT 2..7:
   1. `# retro-check.sh fail-open → retro_bypass JSONL trail` H1.
   2. **The bypass class** — one paragraph (the 3-attempt fail-open is by design; the gap was invisibility).
   3. **The structural fix** — one paragraph naming the JSONL trail + the kind registry doc + the weekly-review consumption.
   4. **The audit signal** — one paragraph (`retro_bypass` rate monotonically decreasing per RLS-PMD review §5.2).
   5. **How to apply** — bulleted: At session start (passive); Weekly cadence (Step 2c sweep); Quarterly (trend).
   6. **See also** — bulleted cross-links to: `.claude/hooks/retro-check.sh` (Task 7), `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md` (Task 8), `.claude/skills/weekly-review/SKILL.md` Step 2c (Task 4), `.claude/rules/pmd-invariants.md` (Task 2), `feedback_lesson_must_pair_with_structural_fix_when_fixable.md`, `docs/research/brehon-rls-pmd-review.md` §4.7 + §5.2.
3. **NO proposal to change the 3-attempt cap** (GOTCHA #1 — fix is TRAIL not cap).
4. **Skip cross-links to absent lessons silently** (GOTCHA #3) — do NOT file blocker.
5. **DQ id discipline** — compute next_id at task start from phase tip's DQ pending+resolved. Likely 316 or 317. Single-task serial.
6. **Worker branch + mid-task push discipline** per `decision-queue.md` "Mid-task visibility".
7. **Attribution integrity**: `from: "impl"` only.
8. **No cargo invocation in the lesson body** — Watchpoint #3.
9. **§16a Story 4 checkpoint** — VALIDATE step probes all exit 0.
10. **Pre-push cargo discipline** per `feedback_fix_impl_pre_push_cargo_check.md`.
11. **Commit subject template** — `feat(rls-r1): add paired retro_bypass lesson (task 9)`. Body cites plan path + commit + Track D scope.
12. **Single commit** per plan §13 norm.
13. **No `--no-verify`**.

## 5. Validation gate (DoD per plan §15)

### 5.1 Output shape

```bash
test -f .claude/lessons/feedback_retro_bypass_governance_log.md
echo "exit: $?"

python3 -c "
import re, yaml
content = open('.claude/lessons/feedback_retro_bypass_governance_log.md').read()
m = re.match(r'^---\n(.*?)\n---', content, re.S)
assert m, 'no frontmatter'
fm = yaml.safe_load(m.group(1))
assert fm['type'] == 'feedback'
forbidden = ['DO use when', 'MUST USE', 'ALWAYS INVOKE', 'CRITICAL']
for kw in forbidden:
    assert kw not in fm['description'], f'forbidden keyword: {kw}'
print('OK')
"

grep -c "retro-check.sh\|governance-log-kinds-jsonl" .claude/lessons/feedback_retro_bypass_governance_log.md   # EXPECT ≥ 2
```

### 5.2 Static analysis (laptop-mode per PRECON-7)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task9-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-rls-r1-task9-check.log
# EXPECT: exit 0
```

### 5.3 §15 validate-pending-laptop DQ (raise after worker push)

Raise DQ entry with computed next_id (likely 316), `kind: "validate-pending-laptop"`, `from: "impl"`, `branch: "<worker-branch>"`, `phase_task: 9`, `commands: ["bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task9-check.log 2>&1; echo exit: $?"]`, `result: null`, all other fields null. Commit + push BEFORE worker exit.

## 6. KNOWN harness limitations

1. **Worker hang post-DQ-raise** — observed twice (Tasks 6, 8). Worker authors output + raises DQ + commits + pushes, then hangs ~0% CPU. Advisor cancels + FF-merges + runs cargo lane-side. EXPECTED: write your file, raise DQ, push, then exit cleanly. Do NOT block on cargo invocation if it hangs >2 min — push first, then attempt cargo, then exit either way.
2. **PMD write fallback** — if `memory_write_eval` fails (FTS5 vtable unavailable in daemon worktree PMD), do NOT block; raise the retro as `kind: "log"`, `from: "impl"`, `answered_by: "impl-self-resolved"` per Tasks 6/8 precedent.
3. **Sensitive-file gate** — `.claude/lessons/**` may block direct Write. Fallback to `/tmp/<file>.md` + `mv` if needed.

## 7. Next steps after this task

- Daemon finalize-merge brings Task 9 commit onto `phase-v1-rls-r1`.
- Advisor polling loop runs cargo-check + mutates DQ to `result: "pass"`.
- Task 10 (dogfood) advances next — three integration probes + report.

## 8. Commit subject template (verbatim)

```
feat(rls-r1): add paired retro_bypass lesson (task 9)
```

Commit body MUST cite:
- Plan path + SHA: `.claude/PRPs/plans/v1-rls-r1.plan.md (97af1b439)`.
- §13 Task 9 implemented.
- Track D paired-lesson scope per plan §4.
- Co-Authored-By trailer.

## 9. DoD for this brief

- [x] Brief committed on `phase-v1-rls-r1` BEFORE Junior task dispatched.
- [x] Brief specifies single-file output + serial chain status.
- [x] Brief cites plan §13 Task 9 by line range (2021-2124).
- [x] Brief lists Required reading per §3 including DQ-id-collision lesson.
- [x] Brief §4 enumerates 13 hard constraints.
- [x] Brief §5 names the validate-pending-laptop DQ shape verbatim.
- [x] Brief §6 carries the KNOWN harness limitation block including hang-post-DQ-raise pattern.

## 10. Commit subject for this brief

`chore(advisor): author Task 9 brief for v1-rls-r1 (serial chain)`
