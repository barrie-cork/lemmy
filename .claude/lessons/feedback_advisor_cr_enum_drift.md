---
name: Advisor/CR enum-value proposals untrusted until PRD-verified
description: If advisor or CR names specific enum variant strings, treat as untrusted until grep'd against PRD + enums.rs. Two-for-two hit rate in v1-JM-a (R5.2 + R5.3).
type: feedback
originSessionId: 211a8c3b-c503-4ae0-afdb-bd47dba4fdc6
---
**Rule:** If the advisor (in a relay) or CodeRabbit (in a finding) names specific enum-value strings or other concrete identifiers (column names, function names, const names), treat those names as **untrusted** until verified against (a) the PRD section + line, and (b) the corresponding `enums.rs` / source-of-truth file. Default assumption: not in the PRD until proven otherwise.

**Why:** Two occurrences in v1-JM-a, both with the same failure mode:
- **R5.2 (cr-9):** advisor relay proposed 5-value enum `reputation_waiver/emergency_panel/sponsor_vouched/admin_override/other` for `JuryConstraintRelaxationReason`. PRD §5.3 + §8.3 actually name 4 values: `SmallPool/ClusterPressure/ClusterPressureExhausted/AdminOverride`. Advisor's values were plausible-sounding placeholders, not the contract. An `Other` variant would have re-opened the ADR-015 free-text leak.
- **R5.3 (cr-4):** CR proposed `Regular/Escalated/Maximum` for `CaseStatusTier`. Actual variants per PRD §4.1:239-242 + `enums.rs:697`: `Founder/Regular/Probation`. Advisor relay echoed CR's claim without cross-check.

Pattern: plausible-sounding domain-adjacent vocabulary ≠ actual contract. Both caught by impl pre-write verification. Neither would have been caught by cargo/tests until PRD-compliance review.

**How to apply:**
- **For advisor relays naming concrete identifiers:** the relay MUST include a `# Source cross-check` section citing PRD §§line AND code-path `file:line`. Missing section = impl treats as untrusted. The advisor cross-checked R10.1 correctly; R5.2 skipped the step.
- **For plans naming enum values, column names, or const names:** every string must cite `PRD §X.Y line Z` or `<file>:<line>` inline. Missing citation = reject at plan review.
- **For impl receiving a relay with named identifiers:** pre-write verification is mandatory. Run `grep -n <identifier> crates/` + read the PRD section. If the relay's named variants don't appear in either, file a clarifying relay back before writing code (R5.2 pattern) or fix inline with a drift note in the commit body (R5.3 pattern — for small doc-only fixes).
- **Retirement condition:** three v1-JM sub-phases with zero occurrences = pattern retired. Until then, this rule is mandatory for every relay + plan review.

Load-bearing quote for future planners and impl: "If the advisor or CR names a specific enum value string, the impl's default assumption should be: not in the PRD until proven otherwise."

**Related:** `feedback_plan_drift_metadata_cross_check.md` (same verification pattern for METADATA keys). Source: v1-JM-a retro §2.2c.
