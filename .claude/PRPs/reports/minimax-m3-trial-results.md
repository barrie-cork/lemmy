# MiniMax M3 vs Sonnet 4.6 — impl-task trial results

Created: 2026-06-12. Follows the runbook at `.claude/PRPs/briefs/minimax-m27-trial-1.md`.

---

## m1-b (2026-06-07) — 2 data points captured post-hoc

See `.claude/PRPs/reports/minimax-ab-m1b-task3-task4.md` for full code diffs and analysis.
Summary recorded here for aggregation.

| Task | Files | Result | Notes |
|---|---|---|---|
| T3 — DTOs | `api_common/src/governance.rs` (1 file) | **Parity** — byte-identical output | MIRROR-heavy DTO; both arms identical on field types, derives |
| T4 — handler+ADR gate | `messaging_config.rs`+`mod.rs` (2 files) | **Sonnet wins** — MiniMax dropped ADR-015 gate + used clippy-denied `.expect()` | Brief ambiguity contributed; T5 proves it was recoverable with tighter brief |

**m1-b conclusion:** MiniMax matches Sonnet on pure pattern-following; drops ADR constraints on handler logic when brief is under-specified. The §2.3a preamble (explain-WHY + explicit-refuse path) and §2.4a load-bearing ADR clause were not in the m1-b brief — they are mandatory in m2-late-2 arms.

---

## m2-late-2 (2026-06-12) — ACTIVE

Trial un-suspended per user directive 2026-06-12. Three qualifying tasks: T3, T4, T5.

**Arm naming:**
- Sonnet control: `ab-test/m2-late-2-t{N}-sonnet` branched from `phase-m2-late-2` tip at T{N} dispatch
- MiniMax arm: `ab-test/m2-late-2-t{N}-minimax` (dispatched via `queue-minimax-task.sh`)

**Per §2.5 two-tier routing:**
- T3 (bridge payload struct + `lookup_by_case`): pure pattern-follow → both arms run
- T4 (handler rewrite, power-level logic): judgment/impl-heavy, no hard ADR gate → both arms run
- T5 (in-module bridge test): test-compile gated → both arms run
- T2 (CR-A atomicity fix): ADR-008 embedded constraint → **Sonnet-only**
- T1 (e2e assertion): modifies `m2_late.rs` → **Sonnet-only** (e2e exclusion criterion 2)

### Task 3 results

> _To be filled at dispatch_

| Metric | Sonnet | MiniMax M3 |
|---|---|---|
| Branch | `ab-test/m2-late-2-t3-sonnet` | `ab-test/m2-late-2-t3-minimax` |
| Cargo first-pass | — | — |
| DQ blockers raised | — | — |
| Commit-shape compliance | — | — |
| Wall-clock (min) | — | — |
| Verdict | — | — |

Notes: _pending_

### Task 4 results

> _To be filled at dispatch_

| Metric | Sonnet | MiniMax M3 |
|---|---|---|
| Branch | `ab-test/m2-late-2-t4-sonnet` | `ab-test/m2-late-2-t4-minimax` |
| Cargo first-pass | — | — |
| DQ blockers raised | — | — |
| Commit-shape compliance | — | — |
| Wall-clock (min) | — | — |
| Verdict | — | — |

Notes: _pending_

### Task 5 results

> _To be filled at dispatch_

| Metric | Sonnet | MiniMax M3 |
|---|---|---|
| Branch | `ab-test/m2-late-2-t5-sonnet` | `ab-test/m2-late-2-t5-minimax` |
| Cargo first-pass | — | — |
| DQ blockers raised | — | — |
| Commit-shape compliance | — | — |
| Wall-clock (min) | — | — |
| Verdict | — | — |

Notes: _pending_

### m2-late-2 aggregate conclusion

> _To be filled after T3/T4/T5 complete_

Running cumulative qualifying total after m2-late-2: **16** (13 prior + 3 this phase).
