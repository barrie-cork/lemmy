---
name: retro-check.sh fail-open emits a JSONL governance-log entry per bypass
description: After 3 attempted Stop hooks without a written retro, retro-check.sh fails open by design (loops are real). v1-rls-r1 adds an additive JSONL trail to .claude/governance-log/retro-bypass.jsonl so the bypass surface is countable and the autonomy-readiness criterion 5.2 (bypass rate monotonically decreasing) is measurable. Bypass behaviour unchanged; the trail is the structural fix.
type: feedback
---

# retro-check.sh fail-open → retro_bypass JSONL trail

## The bypass class

The Stop hook in `.claude/hooks/retro-check.sh` checks for a recent `Task retro:%` row in the PMD `evals` table before allowing a session to exit. If no row is found, the hook **fails open after 3 attempts** — exit 0 with a warning rather than blocking the session indefinitely. This is by design: real loops happen (network blip, MCP server flake, PMD lock, daemon restart mid-write) and a session that cannot exit cannot recover. The 3-attempt fail-open is the right call for autonomy.

The pre-v1-rls-r1 gap was not the fail-open itself — it was **invisibility**. A session that bypassed left no record. The bypass count was unobservable, so calibration-honesty signals (is bypass rate going up? down? stable?) could not be derived. The cap stays; the trail is the structural fix.

## The structural fix

Task 7 of v1-rls-r1 appends `emit_retro_bypass_log` to the fail-open path of `retro-check.sh`. The function writes one JSONL record per bypass to `.claude/governance-log/retro-bypass.jsonl` (gitignored per Task 5; runtime-journal semantics). Each record carries six fields: `timestamp` (ISO 8601 UTC), `session_id` (env-var fallback chain), `attempt_count` (3 at fail-open), `prompt_hash` (16-hex SHA-256 of `CLAUDE_PROMPT`), `branch_at_fail_open` (current git branch), `kind` ("retro_bypass"). The schema is registered in `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md` §2 (Task 8). The consumer is `.claude/skills/weekly-review/SKILL.md` Step 2c retro-harvest sweep (Task 4) — every weekly review scans the prior 7 days of `retro-bypass.jsonl` and surfaces the count.

## The audit signal

Per RLS-PMD review §5.2 (autonomy-readiness criterion): the `retro_bypass` rate per week should trend **monotonically decreasing** over time. Sessions that exhaust the 3-attempt budget represent a confidence-failure: either the PMD plumbing is unreliable (infrastructure debt) or the retro-discipline is slipping (process drift). Either is a calibration signal worth investigating. A rising count is the canary; a stable-low count is the goal; zero is the asymptote.

## How to apply

- **At session start:** nothing required. The trail is passive — the hook writes only when it fails open. No SessionStart wiring needed.
- **Weekly cadence:** weekly-review Step 2c retro-harvest sweep scans `.claude/governance-log/retro-bypass.jsonl` for the prior 7 days; surfaces the count in the weekly summary. Per `feedback_cohort_dq_id_collision.md` and the broader SURFACING-not-auto-promoting discipline, the sweep does not promote bypass entries to lessons or CLAUDE.md automatically. Manual review thereafter.
- **Quarterly:** trend the count week-over-week. If rising, surface as a calibration-honesty signal in the next phase retro. The signal feeds back into how briefs are authored, how cargo wrappers are tuned, how MCP failures are recovered from.

## See also

- `.claude/hooks/retro-check.sh` (Task 7 — the instrumented hook)
- `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md` (Task 8 — the kind registry)
- `.claude/skills/weekly-review/SKILL.md` Step 2c (Task 4 — the consumer)
- `.claude/rules/pmd-invariants.md` (Task 2 — the PMD discipline this trail observes)
- `feedback_lesson_must_pair_with_structural_fix_when_fixable.md` (this lesson IS the paired record)
- `docs/research/brehon-rls-pmd-review.md` §4.7 + §5.2 (originating recommendation)
