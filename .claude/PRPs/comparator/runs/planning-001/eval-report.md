# Eval report — planning-001

**Generated:** 2026-06-07 16:44 UTC
**Judge:** MiniMax-M3 (blind, position-swapped)
**n=1 disclaimer:** this is ONE planning task. Per spec §8, n≥5 required before a binary routing decision.

## 1. Per-dimension divergence

### 1a. Objective code gates

| Gate | Control (Opus) | Challenger (GPT-5.5) | Delta |
|---|---|---|---|
| §-sections present (target 20) | 21 | 17 ▼-4 |  |
| watchpoint artefact cites | 0 | 0 = |  |
| ADR-015 named | 30 | 27 ▼-3 |  |
| ADR-015 callsite | 7 | 4 ▼-3 |  |
| §13 task count | 10 | 9 ▼-1 |  |
| MIRROR refs | 9 | 5 ▼-4 |  |
| §16a story signals | 11 | 0 ▼-11 |  |
| §15 cargo commands | 19 | 8 ▼-11 |  |
| T1-preemption signals | 16 | 0 ▼-16 |  |

### 1b. LLM judge scores (averaged over 2 position-swapped passes)

| Dimension | Control (Opus) | Challenger (GPT-5.5) | Delta |
|---|---|---|---|
| 1. Completeness | 4.0 | 1.0 | -3.0 |
| 2. Watchpoint specificity | 3.0 | 1.0 | -2.0 |
| 3. ADR preservation | 5.0 | 1.0 | -4.0 |
| 4. Task decomposition | 5.0 | 1.0 | -4.0 |
| 5. Story coverage | 2.0 | 1.0 | -1.0 |
| 6. DoD smoke-test | 3.0 | 1.0 | -2.0 |
| 7. T1-preemption | 4.0 | 1.0 | -3.0 |
| **Total (of 35)** | **26.0** | **7.0** | |

## 2. Failure-mode → fix map

Based on code-gate divergence and judge findings.

| Gap | Cause category | Next-run fix |
|---|---|---|
| Missing §-sections | `harness-mismatch — Pi .pi/ prp-plan.md may not enforce the 20-section template as strictly as .claude/commands/prp-core/prp-plan.md` | Add explicit 'Required 20 sections: §1–§16a' enforcement to .pi/prompts/prp-plan.md with a checklist gate before writing the plan |
| Context compaction triggered | `context-pressure — challenger hit 272K threshold during exploration` | Tune context injection: reduce PROJECT_CONTEXT.md verbosity for planning cells; use --no-context-files for a raw-capability pass to isolate harness vs model quality |

## 3. Token/cost breakdown

| Metric | Control (Opus) | Challenger (GPT-5.5) |
|---|---|---|
| Run complete | ✅ (ground truth) | ✅ |
| Wall seconds | n/a (used existing plan) | None |
| Turns | n/a | 43 |
| Tool calls | n/a | 97 |
| Input tokens | n/a | None |
| Output tokens | n/a | None |
| Compaction events | 0 | 1 |

> **Cost note:** control arm runs on Claude Code + Opus subscription (not billed per-token here). Challenger runs via Codex OAuth. True cost comparison requires a common denominator — see spec §4 cost note.

## 4. Replay-ready bundle

Challenger replay bundle at: `runs\planning-001\challenger\replay-bundle`

Contents:
- *(replay-bundle dir not found)*

## Routing recommendation

**Outcome:** `CONTROL_WINS`

Control total: 26.0/35 | Challenger total: 7.0/35

*n=1 — this is a single data point. Extend to n≥5 paired tasks before a production routing flip (spec §8).*
