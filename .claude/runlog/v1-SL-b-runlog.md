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

## bm: triage — 2026-05-07T19:57:05Z
- **PR:** #119
- **Buckets:** fix-in-pr 0 | rebut 1 | carry-forward 1 | done 6 | wont-fix 7
- **Severity breakdown:** critical 0 open / 1 done | major 0 open / 3 done / 1 rebutted / 1 carry-forward | low 0 open / 2 done / 7 wont-fix
- **Comment posted?** yes (https://github.com/barrie-cork/lemmy/pull/119#issuecomment-4400616726)
- **Carry-forward issues filed:** 1 — https://github.com/barrie-cork/lemmy/issues/120 (cr-14 Homebrew paths in .pi/PROJECT_CONTEXT.md)
- **Recommendation:** approve (no fix-in-pr open; no critical open; cr-6 done in a771c49e7; cr-5 deferred to v1-SL-c via documented TODO)
- **YAML:** .claude/PRPs/reviews/pr-119-findings.yaml (counters recomputed; recommendation flipped pending→approve)
- **Comment draft:** .claude/PRPs/reviews/pr-119-comment.md (52 lines, body ready to post)
- **Notes:** poll-#2 re-numbered findings to cr-1..cr-15. cr-5 (was poll-1's `majority_revocation` finding) → rebut citing brief sl-b-fix-impl-3.md §2 deferral. cr-6 (poll-1 cr-7, governance-log schema) → done addressed_in a771c49e7. cr-14 → carry-forward (Pi config out of governance scope). cr-7,cr-9..cr-13,cr-15 → wont-fix (markdown-lint nits / Pi audit artefact).

## bm: merge — 2026-05-07T20:42:04Z
- **PR:** #119 (v1-SL-b — revoke_endorsement handler + DTO + route + 9 tests)
- **base ← head:** governance-v0 ← phase-v1-SL-b
- **merge sha:** 9ae4c332cf15f64a14737f13c7e2ffc4db52889e
- **remote branch deleted?** yes (--delete-branch)
- **trunk position:** 9ae4c332c — Merge pull request #119 from barrie-cork/phase-v1-SL-b
- **findings YAML archived:** .claude/PRPs/reviews/pr-119-findings.yaml
- **adr-compliance note:** advisory red-flag on revoke_endorsement route (known v0 endpoint per ADR-010 §v1); ack comment posted; merged per user instruction
