# m3-core-recording Phase Runlog

## bm-poll-cr — 2026-06-20T05:22:17Z

**Poll 1 on PR #205 (`phase-m3-core-recording → governance-v0`)**

- **CodeRabbit:** 5 findings ingested from inline review comments (1 walkthrough comment + 5 discussion threads)
- **Copilot:** no review posted
- **Findings YAML:** `.claude/PRPs/reviews/pr-205-findings.yaml` created
- **Summary:** 2 open fix-in-pr (cr-4, cr-5), 2 carry-forward candidates (cr-2, cr-3 — scaffold limitations), 1 rebutted (cr-1 — already resolved)

### Finding triage notes

- **cr-1 (DQ timestamp):** rebutted — Task 6 DQ entry already resolved by advisor-laptop (DQ 0aa481cce3a6-001, resolved_at 2026-06-20T05:00:43Z). CR snapshotted stale state.
- **cr-2 (requester pseudonym spoofing):** carry-forward — ADR-015 identity binding deferred to Phase-6. Scaffold limitation per design.
- **cr-3 (empty participant set):** carry-forward — ADR-015 participant-set fetch deferred to Phase-6. Intentional placeholder; all callers return 403 until live session auth wired.
- **cr-4 (fail-open in LiveSink):** fix-in-pr — scaffold methods must return errors until LiveKit egress/S3 upload implemented. Fail-closed gate required.
- **cr-5 (None actor_pseudonym):** fix-in-pr — record_uploaded must validate self.chair before emit. ADR-016 metadata contract + ADR-015 pseudonym binding.

**Recommend:** flag cr-2, cr-3 for advisor gate-3 decision (Phase-6 deferral vs Phase-5 fix). cr-4, cr-5 are fix-in-pr scope.
