---
name: rolling-cumulative-trial-counter
description: Optional experiments needing N data points should use a cumulative counter across phases, not a named-phase gate. Named gates expire silently when the target phase doesn't materialise; cumulative counters don't.
metadata:
  type: feedback
---

Use a **rolling cumulative counter** for any optional trial or experiment that needs N qualifying events but cannot guarantee N events in a single sub-phase.

**Why:** Named-phase gates ("fire at v1-RT-r4") expire silently. RT-r4 was forced to serial dispatch; the trial didn't fire. RT-r5 had only 1 qualifying task. Two sub-phases elapsed without data. A future advisor encountering a stale "fire at v1-RT-r4" note might reset the trial intent entirely rather than recognising the work was still worth accumulating.

**How to apply:**

1. Define qualifying criteria structurally (not as an exclusion list). Example: "not-e2e, MIRROR-ref-heavy, ≤2 files, cargo-gated" — tasks self-exclude by failing a criterion, no manual per-phase exclusion list needed.
2. Keep a **running table** in the trial runbook with one row per task per phase. The cumulative ✅ count is the trigger, not the phase name.
3. Wire a **mechanical check** at the natural decision gate (e.g. plan approval). The check appends rows and reports the running count in the approval surface — zero ongoing overhead.
4. When count reaches the threshold, note it explicitly in the approval surface so the user can confirm before the trial fires.

**Pattern generalises to:** any N-data-point experiment where qualifying events are sparse or unpredictable per phase — model trials, harness benchmarks, latency measurements, feature flag gradual rollouts.

**Instances:**
- MiniMax M2.7 A/B trial — wired 2026-05-31 via §3.5a of `advisor-orchestrator.md` + §3 rolling table in `.claude/PRPs/briefs/minimax-m27-trial-1.md`. Threshold: ≥5 qualifying impl-tasks.
- RT-r4 deferred trial — named-phase gate that expired silently (2026-05-29 → 2026-05-31).
