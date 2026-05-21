## CodeRabbit triage — PR #141 brehon-conformance-audit

Thanks @coderabbitai for the review. Triage summary across the 23 findings on this PR:

**Recommendation: `block`** until the 7 fix-in-pr majors land (one small `chore(skill)` commit per the suggested-diff family).

### Four-bucket counters

| Severity | fix-in-pr | rebut | carry-forward | wont-fix | done | total |
|---|---|---|---|---|---|---|
| critical | 0 | 0 | 0 | 0 | 0 | 0 |
| **major** | **7** | 0 | **1** | 0 | 0 | **8** |
| medium | 0 | 0 | 0 | 0 | 0 | 0 |
| low | 0 | 0 | 0 | 14 | 0 | 14 |
| nit | 0 | 0 | 0 | 1 | 0 | 1 |
| **total** | **7** | **0** | **1** | **15** | **0** | **23** |

### fix-in-pr (7 majors — actionable in this PR)

These are real skill-correctness defects + load-bearing prose fixes; one `chore(skill)` commit per CR's suggested diff.

| ID | File | Fix |
|---|---|---|
| **cr-6** | `.claude/PRPs/briefs/brehon-conformance-audit-impl-10.md:65` | Resolve cargo-check contradiction (§4 #8 vs §5.3 — align with §5.3: workspace cargo-check IS required) |
| **cr-13** | `.claude/PRPs/plans/brehon-conformance-audit.plan.md:1308-1309` | Flip the Task-8a GOTCHA wording — current text contradicts itself (`workspace clippy will STILL fail` vs `tree returns to no-clippy.toml baseline`) |
| **cr-16** | `.claude/skills/brehon-conformance-audit/audit-metrics.schema.json:14-34` | Constrain `ground_truth_compile_caught[]` + `ground_truth_runtime[]` + `predictions[]` items with required-fields + `additionalProperties: false` (CR diff is mechanical) |
| **cr-18** | `.claude/skills/brehon-conformance-audit/METRICS.md:23-26 + :37-39` | Replace `compile_caught[]` / `runtime[]` with `ground_truth_compile_caught[]` / `ground_truth_runtime[]` to match the schema/script contract |
| **cr-19** | `.claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh:110-111, :120-131` | Fix FP attribution — use human-confirmed `false_positives[]` only (per METRICS.md Rule 4); current code infers FP from "prediction without GT" which inflates FP |
| **cr-20** | `.claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh:143-149` | Lead-time loop must iterate `gt_compile + gt_runtime` per METRICS.md spec; currently runtime-only |
| **cr-21** | `.claude/skills/brehon-conformance-audit/scripts/find-sibling.sh:75-77` | Broaden Rust fn regex to accept `pub(crate)`, `const fn`, `unsafe fn`, `extern "C" fn` — all common in Lemmy |

### carry-forward (1 major — follow-up sub-phase)

| ID | File | Why deferred |
|---|---|---|
| **cr-1** | `.claude/decision-queue.json` DQ #307 + #311 | DQ schema-v3 proposal: add `approved_by` / `approved_at` fields on judgment-heavy entries with non-user resolvers. Real coverage gap, but `decision-queue.md` v2 is forward-only and current schema has zero blocking defects (51+ live entries readable). Schema evolution is cross-cutting — needs ADR + retrofit migration. Captured for a future `dq-schema-r1` follow-up. |

### wont-fix (15 — markdownlint nits on advisor meta-docs + 1 DQ-history-rewrite)

- **cr-2** — DQ entry retroactive rewrite. Per `decision-queue.md` v2 forward-only rule, historical idiosyncrasies stay as the audit trail. Future DQ writes should be more disciplined (already in retro §5 W-5).
- **cr-3, cr-5, cr-7, cr-8, cr-9, cr-10, cr-11, cr-12, cr-14, cr-15, cr-17, cr-22** — Markdownlint MD029/MD031/MD040 nits on advisor meta-docs (`.claude/lessons/**`, `.claude/PRPs/plans/**`). No markdownlint enforcement in CI; lessons read correctly in every renderer we use.
- **cr-4** — Markdown table-pipe escape nit on advisor meta-doc.
- **cr-23** — Not a finding; CR placeholder summary line.

### Process note

Two BM Junior workers ran on this PR but neither landed clean:
- bm-poll-cr (#392) pushed to the wrong branch (worker forked off `governance-v0` but pushed to `phase-brehon-conformance-audit` ref it didn't track — no-op). Recovered via cherry-pick of the unreachable commit `dd01153b8`.
- bm-triage (#394) never read its task brief; set every finding to a placeholder `fix-in-pr` with empty rationales. Triage re-done advisor-side.

Both incidents captured for retro promotion (extension of `feedback_bm_false_success_advisor_post_condition_catch.md`).

### Next steps

1. Author one `chore(skill)` commit that lands the 7 fix-in-pr diffs (cr-6, cr-13, cr-16, cr-18, cr-19, cr-20, cr-21) per CR's suggested patches.
2. File a `[brehon-conformance-audit-r1]` GitHub issue for cr-1 (DQ schema-v3 design + ADR).
3. Recompute counters + flip `recommendation: approve`; merge per User Gate 5.
