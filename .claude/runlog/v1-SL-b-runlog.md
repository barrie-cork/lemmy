# Runlog: v1-sponsor-liability-b

## bm: branch cut — 2026-05-04T16:25:27Z
- **branch:** phase-v1-SL-b
- **off:** governance-v0 @ 6c217e0e0
- **plan:** .claude/PRPs/plans/v1-sponsor-liability-b.plan.md
- **next:** impl session takes over for task 1

## bm: triage — 2026-05-07T10:35:00Z
- **PR:** #119
- **Buckets:** fix-in-pr 6 | rebut 14 | carry-forward 0 | done 0 | wont-fix 10
- **Severity breakdown (open in fix-in-pr):** 1 critical (cr-5) | 4 major (cr-4, cr-6, cr-7, cr-22) | 1 nit (cr-23)
- **Comment posted?** dry-run (awaiting user confirm via parent session)
- **Carry-forward issues filed:** 0
- **Recommendation:** block (1 critical fix-in-pr open per SCHEMA.md condition table)
- **Notes:** 14 majors rebutted on `.claude/`/.github/ workflow paths, citing phase-branch.md "Mixed diffs go via PR" + cr-21 workflow logic re-read (line 137 gates on `scan_status`, not `violations`). 10 wont-fix on machine-local marker/audit files (cr-1/2/3/27) and markdownlint hits on advisor/pi meta-prose (cr-24..cr-26, cr-28..cr-30) per fork lint exemption.
## bm: poll-cr — 2026-05-07T19:44:27Z
- **PR:** #119
- **head SHA:** 3f119ce82 (advanced from 8f8ef09a6)
- **CR comments seen:** 15 (0 review / 15 inline / 0 issue)
- **Actionable findings ingested:** 15
- **New findings this poll:** 8 (cr-5 through cr-15)
- **Findings addressed since last poll:** 3 (cr-1, cr-2; cr-3 marked done)
- **Counters:** critical 0 open/1 done/0 rebutted | major 3 open/2 done/0 rebutted | medium 0 open/0 done/0 rebutted | low 7 open/2 done/0 rebutted | nit 0 open/0 done/0 rebutted
- **Recommendation:** pending (3 major fix-in-pr remain)
- **YAML:** .claude/PRPs/reviews/pr-119-findings.yaml (230 bytes)
- **Notes:** Fixes cr-1,cr-2,cr-3,cr-4,cr-8 addressed in commits 2c15df0–b00ae7d; 3 major findings remain open (cr-5 majority_revocation logic, cr-6 governance-log schema, cr-14 hardcoded Homebrew paths); 7 low findings (mostly markdownlint MD040/MD041)
