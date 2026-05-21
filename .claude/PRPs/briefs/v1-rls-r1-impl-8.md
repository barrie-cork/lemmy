# v1-rls-r1 — impl Task 8 brief (serial chain)

## 1. Role + dispatch line

`[role:impl-task] v1-rls-r1 Task 8 — governance-log-kinds JSONL doc + 04 cross-link — see .claude/PRPs/briefs/v1-rls-r1-impl-8.md`

## 2. Scope

Implement plan §13 **Task 8** (lines 1895-2019) of `.claude/PRPs/plans/v1-rls-r1.plan.md` (committed `e8885479b`). One commit, one file created + one modified:

```yaml
creates:
  - docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md
modifies:
  - docs/brehon-law-inspired-network/04-data-model-and-api.md   # one-sentence cross-link near line 405
requires:
  - task: 7
    reason: "The retro_bypass kind registry entry cites Task 7's emit_retro_bypass_log function output format."
```

Create a doc registering hook-emitted JSONL observability kinds (first entry: `retro_bypass` per Task 7). Add a one-sentence cross-link from `04-data-model-and-api.md` near the redaction-service-contract block (line ~405 at HEAD) pointing at the new doc.

**Do NOT** author:
- Any other artifact (Tasks 9, 10, 11, 12).
- A new ADR — observability infra is NOT a design decision per PRECON-5.
- Cross-links FROM the new doc TO `99-decisions-and-open-questions.md` (different systems).
- Any edit under `crates/`, `migrations/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`.

**Branch context:**
- `base_branch` = `phase-v1-rls-r1` (at `e8885479b`).
- Worker forks from `phase-v1-rls-r1` to its own worktree branch.
- One commit on the worker branch; daemon finalize-merges back into `phase-v1-rls-r1`.

## 3. Required reading

### 3.0 Plan + governing brief (P0)

1. `.claude/PRPs/plans/v1-rls-r1.plan.md` §10.6 (architecture vs observability distinction), §13 Task 8 (lines 1895-2019 — IMPLEMENT spec, MIRROR, GOTCHAs, VALIDATE).
2. `.claude/PRPs/briefs/v1-rls-r1-planning-1.md` §0.1.10 — BINDING distinction prose.

### 3.1 Existing artifact MIRRORs (P0 — read BEFORE first Write)

3. `docs/brehon-law-inspired-network/04-data-model-and-api.md` lines 380-430 — locate the redaction-service-contract block / nearest governance-log mention for the cross-link insertion.
4. `docs/brehon-law-inspired-network/03-architecture.md` §6 "The append-only governance log (interface)" — the PG product spec the new doc distinguishes FROM.

### 3.2 Cross-cohort DQ id collision context (P0)

5. `.claude/lessons/feedback_cohort_dq_id_collision.md` — your next_id is **314** (max id at phase tip e8885479b is 313).

### 3.3 Rules (auto-loaded)

6. `.claude/rules/decision-queue.md` — schema v2; impl-task writes `from: "impl"`.
7. `.claude/rules/phase-branch.md` — worker → daemon-finalize → phase branch.
8. `.claude/rules/branch-manager.md` — file ownership.

## 4. Constraints

1. **3 H2 sections** in new doc: `## 1. Scope`, `## 2. Kind registry`, `## 3. Adding a new kind`.
2. **§1 Scope** uses the **verbatim brief prose** (5-line blockquote per plan §13 IMPLEMENT step 2 — copy the brief §0.1.10 prose without paraphrase).
3. **§2 Kind registry** is a 5-column markdown table with columns: `kind | originating hook | sidecar path | JSONL field schema | consumer`. First (only) row is `retro_bypass` per plan §13 IMPLEMENT step 3.
4. **§3 Adding a new kind** has 6 sub-steps (a-f) per plan §13 IMPLEMENT step 4.
5. **04-data-model-and-api.md insertion** uses the **verbatim sentence** from plan §13 IMPLEMENT (file 2 of 2):
   > See also `governance-log-kinds-jsonl.md` for **hook-emitted JSONL observability** — a distinct sidecar trail (e.g. `retro_bypass`) NOT routed through the hash-chained PG `governance_log` table this document describes.
6. **No 99-OQ cross-link FROM the new doc** — separate systems (per PRECON-5).
7. **No "more important than" rhetoric** in the 04 cross-link sentence — neutral framing per Watchpoint #5.
8. **DQ id = 314** (per Required reading #5).
9. **Worker branch + mid-task push discipline** per `decision-queue.md` "Mid-task visibility".
10. **Attribution integrity**: `from: "impl"` only.
11. **No cargo invocation in the doc bodies** — Watchpoint #3.
12. **§16a Story 4 checkpoint** — VALIDATE step probes all exit 0.
13. **Pre-push cargo discipline** per `feedback_fix_impl_pre_push_cargo_check.md`.
14. **Commit subject template** — `feat(rls-r1): add governance-log-kinds JSONL doc + 04 cross-link (task 8)`. Body cites plan path + commit + Track C scope.
15. **Single commit** per plan §13 norm.
16. **No `--no-verify`**.

## 5. Validation gate (DoD per plan §15)

### 5.1 Output shape (per plan §13 Task 8 VALIDATE block)

```bash
test -f docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md
echo "exit: $?"
grep -cE "^## " docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md   # EXPECT 3
grep -c "retro_bypass" docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md   # EXPECT ≥ 2
grep -iE "NOT the v1 product|DIFFERENT THINGS" docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md
grep -c "governance-log-kinds-jsonl" docs/brehon-law-inspired-network/04-data-model-and-api.md  # EXPECT ≥ 1
```

### 5.2 Static analysis (laptop-mode per PRECON-7)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task8-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-rls-r1-task8-check.log
# EXPECT: exit 0
```

### 5.3 §15 validate-pending-laptop DQ (raise after worker push)

Raise DQ entry with `id: 314`, `kind: "validate-pending-laptop"`, `from: "impl"`, `branch: "<worker-branch>"`, `phase_task: 8`, `commands: ["bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task8-check.log 2>&1; echo exit: $?"]`, `result: null`, `log_slice: null`, `failed_commands: null`, `answer: null`, `answered_by: null`, `resolved_at: null`. Commit + push BEFORE worker exit.

## 6. KNOWN harness limitations

1. **Worker hang risk** — observed on Task 6 worker #380 (10+ min idle post-DQ-raise, no cargo invocation, no log output). If this happens: do not retry the same approach; commit + push your output immediately and exit.
2. **Daemon finalize-merge** — operates correctly when daemon's local trunk ref is fresh.
3. **Sensitive-file gate** — `docs/**` is not under `.claude/**`; should not trigger gate.

## 7. Next steps after this task

- Daemon finalize-merge brings Task 8 commit onto `phase-v1-rls-r1`.
- Advisor polling loop runs cargo-check + mutates DQ #314 to `result: "pass"`.
- Task 9 (paired retro_bypass lesson) advances next.

## 8. Commit subject template (verbatim)

```
feat(rls-r1): add governance-log-kinds JSONL doc + 04 cross-link (task 8)
```

Commit body MUST cite:
- Plan path + SHA: `.claude/PRPs/plans/v1-rls-r1.plan.md (e8885479b)`.
- §13 Task 8 implemented.
- PRECON-5 honoured (no new ADR).
- Co-Authored-By trailer.

## 9. DoD for this brief

- [x] Brief committed on `phase-v1-rls-r1` BEFORE Junior task dispatched.
- [x] Brief specifies two-file output + serial chain status.
- [x] Brief cites plan §13 Task 8 by line range (1895-2019).
- [x] Brief lists Required reading per §3 including DQ-id-collision lesson.
- [x] Brief §4 enumerates 16 hard constraints with explicit next_id=314.
- [x] Brief §5 names the validate-pending-laptop DQ shape verbatim.
- [x] Brief §6 carries the KNOWN harness limitation block + hang-risk warning.

## 10. Commit subject for this brief

`chore(advisor): author Task 8 brief for v1-rls-r1 (serial chain)`
