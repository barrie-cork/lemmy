# Handover — v1-RT-r4 CR triage (PR #164) — 2026-05-30

> **UPDATE 2026-05-30 ~10:40Z (post fix-impl-1 + fix-impl-2 dispatch):** Gate 3 PASSED. fix-impl-1 #523 forked STALE base (`10ffbf4e4`); RECOVERED by cherry-picking ONLY code commit `ba709a53d` → tip `972e27ccf`. `cargo check --workspace --features full` = **CHECK_EXIT_0**. fix-impl-1 applied **cr-1 ✓, cr-4 ✓ (+bonus delete-count race guard), cr-5 ✓** but **SILENTLY DROPPED cr-2 (dup-guard) + cr-3 (scrub) + cr-7** (caught by advisor diff-vs-enumeration spot-check — grep confirmed 0 occurrences of sponsor_allowlist_exists/scrub in handler). governance.rs +4 = benign comment-only ADR-010 carve-out. RESPONSE: FF'd daemon local phase to `972e27ccf` then pushed fix-impl-2 brief → daemon FF'd to **`8de4f1a2c`** → dispatched **fix-impl-2 #524** (cr-2+cr-3+cr-7, 2 files, §2.0 HARD base-check). NEXT: poll #524 → cherry-pick its fix commit (NOT the DQ churn) → laptop check+clippy+e2e → resolve worker DQs on tip as advisor-laptop → push → re-poll CR (set addressed_in) → retro → merge gates. Do NOT merge worker DQ commits (stale-base 1207-line decision-queue.json churn).

**Lane:** Mode A, CWD `brehon-fork-rt-r4`, branch `phase-v1-RT-r4` @ `9f3c2a1b8` (was 8abec2d1f).
**Stage:** fix-impl-1 cherry-picked; validating on laptop; pre-push.
**bm-pr task #522:** DONE → opened **PR #164** (base governance-v0, head phase-v1-RT-r4, OPEN, not draft).

## What's done this session
- Verified lane (rt-r4 + redaction-r1 + quality-r2-validate worktrees active; only this session driving rt-r4).
- Corrected pre-compaction error: PR is **#164** not #163; bm-pr #522 succeeded.
- Read ALL **16 CR line findings** (NOT 6 as summary claimed). Dumped to `.claude/PRPs/debug/pr164-line-comments.json` (UTF-16) + `pr164-issue-comments.json`.
- **Every claim verified against actual source** (per feedback_verify_automated_reviewer_claims_against_compiler.md).
- Wrote `.claude/PRPs/reviews/pr-164-findings.yaml` (canonical SCHEMA.md shape, 12 deduped findings) + `pr-164-comment.md` (triage digest). Both gitignored.

## Verified verdicts (12 findings, deduped)

**3 open MAJORS (fix-in-PR, all real, all fixable without migration):**
- **cr-1** `create_endorsement.rs:183` — AgeOrSurety `.is_err()` swallows DB/transient errors → may admit a sponsor that should be rejected (silent-failure). Fix: match `NotFound` age-denial specifically; propagate other errors. ~4 lines.
- **cr-2** `admin_sponsor_allowlist.rs` add — UNIQUE(community_id,person_id) + nullable community_id (r1, NO partial unique idx) → Postgres NULL-distinct lets add() insert dup instance-wide rows; remove() deletes one; log double-emits. Fix in-handler: `sponsor_allowlist_exists` check before insert. ~5 lines. Migration confirmed: 2026-04-22 up.sql:6 UNIQUE; 2026-05-10 up.sql:11 drop NOT NULL, no partial idx.
- **cr-3** `admin_sponsor_allowlist.rs:82` — user `note` logged raw into governance payload; .coderabbit.yaml:108-110 + ADR-015 require scrub() on every user-visible string in log payloads. **CAVEAT: confirm scrub() helper exists on this branch + import path FIRST** (redaction module touched by concurrent v1-redaction-r1 lane). If no helper reachable → downgrade to carry-forward + blocker DQ, do NOT invent one. (Was mid-search for scrub/redact in crates when context ran low — FINISH THIS CHECK before authoring fix.)

**2 mediums + 1 low fix-in-PR (cheap, bundle):** cr-4 (remove() `Unknown`→`NotFound`, :155, 1 line), cr-5 (e2e `assert!(result.is_err())` at e2e.rs:18207/18273/18333 → assert NotFound shape once cr-1 lands; fold cr-6/F6 remove-test row-deletion assertion in), cr-7 (`sponsor_allowlist_exists` materializes full row, sponsor_allowlist.rs:86,95 → `select(exists(...))`).

**1 REBUT (false positive):** cr-6 ADR-010 "exactly 11 endpoints" — `.coderabbit.yaml:93-99` scopes that check to **v0/Phase 3**. ADR-010 (99-decisions:134) = v0 only; v1 separate release. PRD:181 names /sponsor-allowlist/add+remove in-scope; PRD:335 "v1.r4: add sponsor_allowlist admin endpoints"; OQ-020 ratified strategies. **NOT ADR-affecting, NOT catch-fire.**

**2 carry-forward:** cr-9 (fix stale `.coderabbit.yaml` rule — root cause of cr-6 FP; direct-on-trunk meta-work, low pri), cr-12 (e2e HTTP-path coverage — suite-wide pattern, not r4 regression; brief §3.3 prescribed direct handler calls).

**1 wont-fix:** cr-8 (Reputation arm query dedup, create_endorsement.rs:198-215 — premature abstraction, 1 call site).

**2 advisor-side mechanical (NOT impl-task):**
- cr-10 — DQ entries `77a6a1be5dc4-001` + `3c87676f024d-001` have `timestamp:10:00:00Z` placeholder LATER than their `resolved_at` (02:44/04:29). My own validate-pending-laptop mutations. Correct the 2 timestamps on phase branch.
- cr-11 — runlog MD022 blank-line nit.

## Recommended path (pending user decision at gate 3)
One fix-in-PR impl-task bundling cr-1+cr-2+cr-3(if scrub reachable)+cr-4+cr-5+cr-7 — 3 files (create_endorsement.rs, admin_sponsor_allowlist.rs, e2e.rs). Advisor does cr-10+cr-11 directly. Re-validate check/clippy/e2e. No CR rebut comments posted (private repo).

## NEXT ACTIONS (in order)
1. Finish scrub() helper existence check (git grep scrub/redact in crates/) → decides cr-3 in-or-out.
2. Surface gate-3 four-bucket summary + the scrub caveat to user via AskUserQuestion. WAIT.
3. On approval: author fix-impl brief on phase branch (Mode A — commit + push origin phase-v1-RT-r4). Dispatch [role:impl-task]. Per feedback_fix_impl_pre_locate_e2e_anchors.md (e2e edits) + feedback_lemmy_error_no_std_error.md.
4. Advisor: fix cr-10 (DQ timestamps) + cr-11 (runlog) on phase branch.
5. After fix lands: validate-pending-laptop (check+clippy+e2e) → mutate DQ pass.
6. Re-poll CR (bm-poll-cr) to confirm addressed_in SHAs.
7. merge-forward check (git log origin/governance-v0 ^phase-v1-RT-r4).
8. Task 8 retro (gates gate 6). Gate 5 merge confirm → bm-merge. Gate 6 retro → /brehon-phase-transition.

## Gotchas this session
- **Bash-tool CWD resolves to brehon-fork (canonical) intermittently post-/compact** — use PowerShell for git, OR `cd "C:/Users/barri/Developer/brehon-fork-rt-r4" &&` prefix every Bash.
- PowerShell mangles inline `gh --jq` with gsub/spaces — use `--%` stop-parsing or dump to file + Python (utf-16 decode for PS redirects!).
- 6→16 finding discrepancy: CR's "Actionable comments posted: 6" headline ≠ total line comments. Always count `gh api .../pulls/164/comments | length`.
- roadmap.json shows `v1-RT-r4: unstarted` (stale; we shipped it) — cosmetic, fix at transition.
- Duplicate worktree `brehon-fork-rt-r4-r4` may exist (same tip) — clean at ship.
