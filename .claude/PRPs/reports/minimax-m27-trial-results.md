# MiniMax M2.7 vs Sonnet 4.6 — impl-task A/B trial — RUNNING (m1-b)

**Status:** EXECUTING on m1-b (Tree B). First arms ran 2026-06-04.

**Decision rule (from runbook §2.5):** analyse after **5 cumulative eligible tasks, cross-phase** (user-confirmed 2026-06-04). m1-b contributes 3 (Tasks 3/4/5); the remaining 2 come from the next MIRROR-heavy phase. **Do NOT decide at end of m1-b** — n=3 is below the threshold.

**Trial model (user-confirmed 2026-06-04):**
- Both arms run in **parallel** off the same phase-branch base, on throwaway `ab-test/{sonnet,minimax}-impl-<N>` branches.
- The **Sonnet (control) arm is canonical** — only it merges into `phase-m1-b`. The **MiniMax arm is comparison data only and NEVER merges.**
- MiniMax pinned to **M2.7** (not M2.5). Dispatched via `queue-minimax-task.sh` (or the inline `junior task add` SSH equivalent); control via MCP `create_task`.

**Prior status:** the v1-RT-r4 arming (2026-05-30) never ran — serial dispatch after an OOM left no parallel arm. That stub is superseded by this file.

---

## Comparison table

| Task | Metric | Sonnet 4.6 (control) | MiniMax-M2.7 (trial) | Notes |
|---|---|---|---|---|
| **T3** (DTOs) | Junior id | #576 | #577 | base `phase-m1-b@81762a740`; brief `m1-b-impl-3.md` |
| | Wall-clock | 2m22s (10:36:47→10:39:09) | **1m39s** (10:38:37→10:40:16) | MiniMax faster |
| | DQ blockers raised | 0 | 0 | tie — neither needed clarification |
| | Cargo first-attempt | **PASS** — `check --workspace --features full` Finished 3m22s, 0 err 0 warn; `e2e --no-run` Finished 21m57s, `E2E_COMPILE_EXIT_0` | **PASS (inferred)** — code byte-identical to #576; the Sonnet compile (both commands green) proves it | see note below |
| | Commit subject | `feat(api_common): … (task 3)` ✓ | `feat(api_common): … (task 3)` ✓ | both conform |
| | HANDOVER trailer | ✓ complete | ✓ complete (identical keyDecisions) | both conform |
| | LESSON trailer | none | ✓ (correct ts-rs/Eq note) | MiniMax added a useful lesson |
| | Diff quality | correct; terse docs; backtick route refs match sibling | correct; **richer docs** (explains value_type derivation + scope format); dropped route backticks (minor sibling-style drift) | both honored the load-bearing `{previous,new}` divergence (no governance_log_id/preview/applied); both correctly omitted `Eq` |

**Cargo-validation note (T3):** the two arms produced **byte-identical struct definitions** — the ONLY diff is in doc-comment prose, which does not affect compilation. The canonical (Sonnet) arm is validated authoritatively on the laptop (`cargo check --workspace --features full` + `e2e --no-run`); the MiniMax arm's compile result is inferred-identical rather than re-run, to avoid a second serial ~9-min cargo pass on the same `target/` for provably-equivalent code. This inference is recorded explicitly so the n=5 decision isn't built on a hidden assumption.

### T3 qualitative read

Both arms are **functionally equivalent and correct on the first attempt.** Both honored the brief's load-bearing divergence (simple `{ previous, new }` response, NOT the full sibling shape) and the `serde_json::Value`-is-not-`Eq` constraint. MiniMax was ~33% faster wall-clock and volunteered a correct `LESSON:` trailer; Sonnet's commit-body prose and doc-comment backtick style matched the codebase conventions marginally more closely. **No quality gap that would move the cutover decision either way at n=1.**

---

## Pending (this phase)

- **T4** (admin handler, single write) — MiniMax-eligible; arms dispatch after T3 closes.
- **T5** (identity-policy validator + routes) — MiniMax-eligible; arms after T4.

## Pending (cross-phase, to reach n=5)

- 2 more eligible tasks from the next MIRROR-heavy sub-phase.

## Decision (deferred until n=5)

Per runbook §2.5: M2.7 matches Sonnet on cargo-first-pass AND DQ-blocker-rate across the 5 tasks → switch impl-task to MiniMax (≈10× cost saving). M2.7 worse on either → stay Sonnet. Mixed at n=5 → extend to n=8-10 or stay Sonnet (conservative default).

**Running tally (n=1):** cargo-first-pass tie (both pass), DQ-blocker tie (0/0), wall-clock MiniMax-favoured, quality tie. Too early to read.

---

_Live results file; appended per eligible task. Authored by advisor during m1-b 2026-06-04._
