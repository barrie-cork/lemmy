---
name: outcome-not-cause-check-retro-bypass
description: When an orchestration anomaly appears, check retro-bypass.jsonl first before diagnosing code or DQ causes — hook fail-open is often the proximate cause.
metadata:
  type: feedback
---

# Outcome ≠ cause: check retro-bypass.jsonl first

## Problem

When an orchestration anomaly appears (task reports done, retro missing, branch in unexpected state), the natural assumption is a process error or code bug. In 2× confirmed incidents, the actual cause was the Stop hook's fail-open path — the hook ran out of retries, silently bypassed the retro requirement, and the task finished without a retro write. The missing retro looked like a process failure but was a hook mechanics failure.

## Rule

Before diagnosing any orchestration anomaly:

1. Check `.claude/governance-log/retro-bypass.jsonl` for a recent `retro_bypass` record matching the task's branch and approximate timestamp.
2. If a bypass record exists: the hook failed, not the process. Investigate the hook failure (`retro-check.sh` exit code, MCP availability) before any DQ or fix-impl dispatch.
3. If no bypass record: the anomaly is a genuine process failure — proceed with normal DQ/RCA path.

**Why:** The Stop hook is load-bearing for the RLS feedback loop. Its fail-open path (3-attempt cap) is intentional (avoid infinite loops) but makes its bypasses invisible to the advisor unless explicitly checked. Checking the JSONL takes 5 seconds; skipping it costs ~30 min of misdirected diagnosis.

## See also

- `feedback_retro_bypass_governance_log.md` — JSONL schema + rate-trend audit
- `advisor-orchestrator.md §5.5` — retro-bypass observability
