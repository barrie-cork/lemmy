# Brehon conformance-audit metrics

This document is the ground-truth reference for the brehon-conformance-audit skill's
self-improving metrics loop. It codifies the four ground-truth attribution rules that
determine what counts as a true/false positive or negative, the four metrics formulas,
the three calibration cadences, a worked example (populated by the Task 7 dogfood), and
the process for adding a new metric.

## Per-run journal shape

Every skill invocation writes a machine-readable per-run metrics file at:

```
.claude/PRPs/audit-metrics/<scope-slug>.json
```

The JSON schema for this file is defined at:

```
.claude/skills/brehon-conformance-audit/audit-metrics.schema.json
```

At invocation time the skill writes `predictions[]` — one entry per flagged axis per
function. After §15 cargo validation, CodeRabbit triage, and any post-merge bug fix, the
corresponding `compile_caught[]` and `runtime[]` ground-truth fields are mutated by hand
(or by the advisor session running `compute-metrics.sh`). This two-phase write is how
the skill accumulates precision/recall data over time without requiring live compiler
access from within the skill body.

Key fields in the per-run record:

| Field | Written by | When |
|---|---|---|
| `run_at` | skill at invocation | immediately |
| `scope` | skill at invocation | immediately |
| `predictions[]` | skill per flagged finding | immediately |
| `compile_caught[]` | advisor / user | after §15 cargo run |
| `runtime[]` | advisor / user | after CR triage or post-merge fix |
| `false_positives[]` | advisor / user | at retro (human verdict required — Rule 4) |

## Four ground-truth attribution rules

These rules determine which events count as true positives, false positives, or false
negatives when computing precision and recall. They are binding — do not modify them
without a formal lesson-promotion at the every-major-version calibration cadence.

**Rule 1 — §15 failures count only with axis attribution.**
`error[E####]` maps unambiguously to one of the six axes. Multi-axis errors count once
per axis. Out-of-axis failures (unrelated compile errors, formatting errors, etc.) do NOT
penalise recall.

**Rule 2 — CR findings count only with severity + bucket + sibling-conformance reference.**
A CodeRabbit finding counts toward recall only when ALL three conditions hold:
`severity ≥ major` AND `bucket = fix-in-pr` AND the finding text explicitly references
sibling-conformance (i.e. cites a same-file sibling or the Phase-6 convention-divergence
defect class). CR findings that lack sibling-conformance context are not counted.

**Rule 3 — Post-merge bugs count only with same-file axis-pattern modification within 30 days.**
A post-merge bug counts toward recall only if the fix-commit diff modifies an
axis-relevant pattern in the same file within 30 calendar days of the original merge.
Bugs fixed later than 30 days, or whose fix touches a different file, are not attributed
to the audit.

**Rule 4 — False positives confirmed only by human verdict at retro.**
A prediction is recorded as a false positive only after a human reviewer (advisor or
user) explicitly confirms it at retro time. Automated tools do not confirm false
positives. The confirmation is written into `false_positives[]` with the reviewer's
initials and the retro date.

## Four metrics formulas

These formulas are computed by `scripts/compute-metrics.sh` from the per-run JSON files.
Definitions follow Rules 1–4 above.

**Precision per axis:**

```
precision per axis N = true_positives_N / (true_positives_N + false_positives_N)
```

Where true positives are predictions confirmed by a ground-truth event (Rules 1–3) and
false positives are predictions confirmed as incorrect by human verdict (Rule 4).

**Recall per axis:**

```
recall per axis N = true_positives_N / (true_positives_N + false_negatives_N)
```

Where false negatives are ground-truth events (from Rules 1–3) that had no corresponding
prediction. Predictions without any ground-truth event are not false negatives (per Rule 1:
out-of-axis failures do not penalise recall).

**Lead time:**

```
lead time = wall_clock(skill_run_at) - wall_clock(ground_truth_event_date)
```

Negative lead time = the skill flagged the issue before the ground-truth event (good).
Positive lead time = the skill flagged it after (audit found it late, or as a
retrospective catch). Reported as median over the corpus. Measured using
`git log -1 --format=%aI <evidence_commit_sha>` for `compile_caught[]` and `runtime[]`
entries, compared against the `run_at` ISO timestamp in the metrics file.

**Latent-footgun catch rate:**

```
latent-footgun catch rate = count(axis-4 predictions where runtime[] exists AND compile_caught[] is empty)
```

Counts axis-4 findings that the skill caught but the compiler did not. This is the
headline metric for the skill's value: each non-zero count represents a latent footgun
that would have survived `cargo check` and reached production. Computed at retro time
once `compile_caught[]` and `runtime[]` fields are populated.

## Three calibration cadences

**Per-sub-phase (every retro):**
After each sub-phase retro, run `compute-metrics.sh` on the phase's metrics file:

```bash
bash .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh \
  .claude/PRPs/audit-metrics/<phase-slug>.json
```

Write the precision/recall summary and latent-footgun catch rate into retro §5
watch-items. Flag any axis whose precision drops below 0.5 or recall drops below 0.7 for
review at the next every-3-sub-phases cadence.

**Every 3 sub-phases:**
Aggregate across the last 3 metrics files:

```bash
bash .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh \
  .claude/PRPs/audit-metrics/phase-A.json \
  .claude/PRPs/audit-metrics/phase-B.json \
  .claude/PRPs/audit-metrics/phase-C.json
```

Trend per-axis precision and recall across the three phases. Identify drift: an axis
whose precision is consistently below 0.6 across 3 phases is generating noise and
warrants detection-command refinement. An axis whose recall is consistently below 0.7
is missing real findings and warrants sibling-locator or grep refinement in the
corresponding `axes/N-*.md` detection spec.

**Every Brehon major version:**
Review the six-axis schema holistically. Assess whether a candidate axis-7 has
accumulated 3+ instances in the lessons corpus warranting promotion. New axes are added
ONLY at this cadence — never ad-hoc. The promotion process:

1. Identify 3+ lesson files or PMD memories recording the same defect pattern.
2. Draft the axis sub-file (`axes/7-<name>.md`) following the existing axis format.
3. Add a row to `SKILL.md` §"Six axes".
4. Adjust precision/recall targets in this document.
5. Commit with subject `feat(skill): promote axis-7 <name> (every-major-version cadence)`.

Also review Tier-1/2/3 thresholds: if Tier-2 findings are consistently confirmed as
real (precision rising toward Tier-1 territory), consider upgrading. If Tier-1 findings
are frequently confirmed as false positives (precision < 0.6 for 3 sub-phases), consider
downgrading.

## Worked example

_Populated by Task 7 dogfood backfill._

This section will contain the worked example from the dogfood run against
`v1-federation-inbound-b` at the pre-fix-impl-3 SHA. The expected outcome:

- Input: `phase-diff` on the `v1-federation-inbound-b` branch at the pre-fix-impl-3 tip.
- Finding 6.1: axis-4 prediction, `Tier 1`, on `receive_remote_moderation_label` at
  `crates/apub/activities/src/governance/inbox.rs:~735`.
  Evidence: sibling four lines away hard-errors via `.domain().ok_or_else(...)`.
  Prediction: `.unwrap_or_default()` would persist `source_instance = ''` for a
  domainless remote actor.
- Ground truth (post-fix-impl-3 merge): `runtime[]` entry confirming the fix at SHA
  `8b04e69a6`.
- Computed metric: `recall per axis 4 = 1/1 = 1.0` (skill caught the only known instance).
  `latent-footgun catch rate = 1` (compiler passed; skill caught).

**Task 7 will Edit this section to replace the above with real numbers from the actual
dogfood run.**

## Adding a new metric

New metrics are proposed at the every-major-version calibration cadence (see §"Three
calibration cadences" above). They are never added ad-hoc.

The process:

1. **Evidence requirement:** at least 3 retro §5 watch-items or PMD memories recording
   the gap the new metric would fill. Document the evidence commit SHAs or PMD memory IDs
   in the lesson file that triggers promotion.
2. **Formula draft:** define the metric in the same format as §"Four metrics formulas"
   above — numerator, denominator, domain, null-case handling.
3. **Attribution rule extension:** add a Rule 5+ if the metric requires a new category
   of ground-truth event beyond Rules 1–4.
4. **Schema update:** add the corresponding fields to `audit-metrics.schema.json` and
   update `scripts/compute-metrics.sh` to compute and report the new metric.
5. **Calibration target:** set an initial precision/recall target (or equivalent) that
   would trigger review if missed for 3 consecutive sub-phases.
6. **Commit:** `feat(skill): add <metric-name> metric (every-major-version cadence)`.

Do not add a metric just because a finding class is interesting. Metrics exist to
detect drift in the skill's own quality; add one only when there is documented evidence
of a gap that a new metric would close.
