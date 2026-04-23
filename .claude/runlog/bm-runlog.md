# Branch-manager runlog

Append-only ledger of BM-session state-changing actions. Each entry is
prefixed `bm:` and timestamped UTC. Created 2026-04-23.

---

## bm: branch cut — 2026-04-23T17:15:00Z
- **branch:** chore/v1-AD-wrap-up
- **off:** governance-v0 @ dbc0fecad
- **plan:** n/a — chore branch (wrap-up for v1-AD-a..d retro + DQ cleanup)
- **carry-forward from trunk:** `.claude/decision-queue.json` (modified, 4 pending→0) + `.claude/PRPs/reports/v1-AD-meta-retro.md` (new, 353 lines)
- **pushed?:** No (deferred to first commit + `/bm-push`)
- **next:** impl session stages + commits carry-forward, then `/bm-push` + `/bm-pr`

---

## bm: merge — 2026-04-23T16:51:31Z
- **PR:** #87 — Phase v1-AD-d — Dashboard aggregate + SSE audit stream
- **Base ← Head:** `governance-v0` ← `phase-v1-AD-d` @ `83e0dfdb7`
- **Merge commit:** `092a67208cac67cfc041770e4d91082cc97babb9`
- **Method:** `--merge` (task-per-commit history preserved; 18 commits ff-merged)
- **Delete-branch:** YES (`origin/phase-v1-AD-d` removed)
- **Digest comment:** https://github.com/barrie-cork/lemmy/pull/87#issuecomment-4306198672
- **Final counters:** fix-in-pr 0 | done 10 | rebut 5 | carry-forward 11 | wont-fix 0 (total 26, all CR)
- **Open carry-forward items:** issue #88 (cr-1..cr-11, BM tooling follow-ups)
- **Local trunk:** fast-forwarded `1e6cc14dd..092a67208` (25 commits)
- **Stashed working-tree carry-forward:** `.claude/runlog/bm-runlog.md` + `.gitignore` (BM-owned per branch-manager.md)

---

## bm: triage (run #4) — 2026-04-23T16:15:00Z
- **PR:** #87
- **head SHA:** 83e0dfdb7
- **Promotions (fix-in-pr → done):** cr-21 (low, `8b99a3401`), cr-22 (major, `8b99a3401`), cr-23 (major, `7ea0844cd`), cr-24 (major, `7ea0844cd`), cr-25 (low, `357dca6d4`), cr-26 (critical, `83e0dfdb7`)
- **SHA verification:** all 4 unique SHAs (`8b99a3401`, `7ea0844cd`, `357dca6d4`, `83e0dfdb7`) present in `git log governance-v0..phase-v1-AD-d`
- **Buckets after triage:** fix-in-pr 0 | rebut 5 | carry-forward 11 | done 10 | wont-fix 0 (total 26)
- **Counters (severity × bucket):** critical done 1 | major done 5 rebutted 5 carry_forward 9 | low done 3 carry_forward 2 | nit done 1
- **Recommendation:** `block` → `approve` (zero fix-in-pr rows, zero open critical)
- **Rebuttals untouched:** cr-12, cr-16, cr-17, cr-19, cr-20 (all major, all with citations from triage #1)
- **Carry-forward untouched:** cr-1..cr-11 (all consolidated in issue #88)
- **Comment posted?** HOLD — parent (impl) session will gate the post via AskUserQuestion
- **Comment draft regenerated:** `.claude/PRPs/reviews/pr-87-comment.md` (fresh disposition table; annotated "post-poll #7 + triage #4")
- **Next suggested:** parent asks user to post digest comment → `/bm-merge 87` on confirm

---

## bm: poll-cr (no-op) — 2026-04-23T16:05:00Z
- **PR:** #87
- **head SHA:** 83e0dfdb7 (unchanged since poll #7 at 15:59:19Z)
- **CR comments seen:** 31 (5 review / 25 inline / 1 issue) — totals identical to poll #7
- **Actionable findings ingested:** 26 (no new)
- **Latest CR review:** #4163744368 at 2026-04-23T15:27:20Z against commit 357dca6d4 (cr-26) — still no re-review of 83e0dfdb7
- **Walkthrough `updated_at`:** 2026-04-23T15:49:45Z (predates poll #7; no edit since)
- **Action per Phase 5.1:** SKIP-THE-WRITE short-circuit (head SHA unchanged + no new CR comments). YAML untouched; `poll_count` stays at 7; `last_poll_at` stays at 2026-04-23T15:59:19Z.
- **Counters (unchanged from poll #7):** critical 1/0/0 | major 3/2/5 | medium 0/0/0 | low 2/1/0 | nit 0/1/0
- **Recommendation (unchanged):** block
- **Notes:** This is poll #8 in the fix-session chain; CR still has not picked up 83e0dfdb7 (~40 min since push at 15:48:00Z). Six fix-in-pr rows remain eligible for triage promotion to `done` (cr-21, cr-22, cr-23, cr-24, cr-25, cr-26) — parent session may choose to either wait longer for CR's silent-approval re-review, OR invoke `/bm-triage 87` now to promote based on addressed_in SHAs already set.

---

## bm: poll-cr — 2026-04-23T15:56:00Z
- **PR:** #87
- **head SHA:** 83e0dfdb7 (changed since last poll — advanced from 357dca6d4)
- **CR comments seen:** 31 (5 review / 25 inline / 1 issue)
- **Actionable findings ingested:** 26 (all prior; 0 new from walkthrough/pre-merge)
- **New findings this poll:** 0 (CR has not re-reviewed head 83e0dfdb7 yet; last CR review was #4163744368 at 15:27:20Z against 7ea0844cd..357dca6d4)
- **Findings addressed since last poll:** 1 (cr-26 → addressed_in: 83e0dfdb7 via commit-subject SHA match; bucket stays fix-in-pr until triage)
- **Counters:** critical 1/0/0 (open/done/rebutted) | major 3/2/5 | medium 0/0/0 | low 2/1/0 | nit 0/1/0
- **Recommendation:** block (schema-strict: cr-26 is critical + bucket=fix-in-pr; recommendation flips on triage promotion to done)
- **YAML:** .claude/PRPs/reviews/pr-87-findings.yaml
- **Notes:** All 5 prior addressed_in SHAs verified present (no force-push). Six eligible-to-promote rows pending triage: cr-21, cr-22, cr-23, cr-24, cr-25 (all have addressed_in set from prior polls) + cr-26 (set this poll). Recommended next step is `/bm-triage 87` once user confirms CR re-review lands on 83e0dfdb7, OR user may triage-promote the 6 fix-in-pr rows now if confident the addressed_in commits genuinely close the findings.

---

## bm: push — 2026-04-23T15:48:00Z
- **branch:** phase-v1-AD-d
- **commits pushed:** 1 (357dca6d4..83e0dfdb7, fast-forward)
  - 83e0dfdb7 test(admin-gate): borrow error_type in matches! (cr-26)
- **remote ref:** origin/phase-v1-AD-d @ 83e0dfdb7
- **upstream tracking:** already set
- **context:** Critical cr-26 fix — add `&` to `matches!(err.error_type, ...)` at e2e.rs:5901 and 6286 (admin_dashboard_forbidden_for_non_admin + admin_audit_stream_forbidden_for_non_admin). Existing code compiled, but defensive fix per CR; block-merge gate per feedback_coderabbit_block_merge_critical.md. Validation: both e2e targets passed; clippy --workspace --features full --no-deps -- -D warnings exit 0.
- **PR status:** PR #87 open; remote head now at 83e0dfdb7. CodeRabbit will re-review on push; should close the last Critical and all 9 fix-in-pr items on the triage pass.
- **next:** /bm-poll-cr 87 (~5 min) → /bm-triage 87

---

## bm: push — 2026-04-23T15:21:45Z
- **branch:** phase-v1-AD-d
- **commits pushed:** 1 (7ea0844cd..357dca6d4, fast-forward)
  - 357dca6d4 test(admin-dashboard): tighten status-count assertions (cr-25)
- **remote ref:** origin/phase-v1-AD-d @ 357dca6d4
- **upstream tracking:** already set (from prior push)
- **context:** test-only tighten — `>= 1` → `assert_eq!(..., Some(1))` on 3 assertions in `admin_dashboard_aggregates_populated_data`. Closes cr-25 (low, last open fix-in-pr finding). No production code touched. Validation: e2e target test passed with tightened asserts; clippy --workspace --features full --no-deps -- -D warnings exit 0.
- **PR status:** PR #87 open; remote head now at 357dca6d4. CodeRabbit will re-review on push.
- **next:** /bm-poll-cr 87 (expect 0 open fix-in-pr on all severities once CR re-review lands)

---

## bm: poll-cr — 2026-04-23T15:31:00Z
- **PR:** #87 (phase-v1-AD-d → governance-v0)
- **head SHA:** 357dca6d4 (changed since last poll? yes — 7ea0844cd → 357dca6d4)
- **CR comments seen:** 31 total (5 review / 25 inline / 1 issue)
- **Actionable findings ingested:** 26 cumulative (1 new this poll, 0 from walkthrough/pre-merge)
- **New findings this poll:** 1 (cr-26, Critical 🔴 at e2e.rs:5902 — `matches!(err.error_type, ...)` move-after-move; fix is borrow `&err.error_type` at both call sites 5901 and 6286)
- **Findings addressed since last poll:** 1 (cr-25 → 357dca6d4; heuristic SHA-match on commit subject `(cr-25)`; bucket stays fix-in-pr pending triage)
- **Counters:** critical 1/0/0 | major 3/2/5 (open/done/rebutted; carry-forward 9) | medium 0/0/0 | low 2/1/0 (carry-forward 2) | nit 0/1/0
- **Recommendation:** request-changes (new CRITICAL finding cr-26 — block-merge per feedback_coderabbit_block_merge_critical.md)
- **YAML:** .claude/PRPs/reviews/pr-87-findings.yaml (22234 bytes, 26 findings)
- **Notes:** CR review #4163744368 (submitted 15:27:20Z) re-reviewed diff 7ea0844cd..357dca6d4 and posted exactly 1 actionable finding: the critical move-after-move bug on the cr-13/cr-18/cr-25 assertion pattern. No follow-ups on cr-25 itself. The fix-in-pr ladder now: cr-21/cr-22 (addressed_in 8b99a3401), cr-23/cr-24 (addressed_in 7ea0844cd), cr-25 (addressed_in 357dca6d4) all pending triage → done promotion; cr-26 NEW (addressed_in null, needs impl to fix). Expected behaviour flip: CRITICAL cr-26 becomes the new gating finding for merge. 5 rebuttals + 11 carry-forwards untouched this poll. No outside-diff findings this round (e2e.rs is in diff). No force-push. Write-only to YAML + runlog per verb spec; no comment drafted.

---

## bm: poll-cr — 2026-04-23T01:50:00Z
- **PR:** #79 (chore/rules-housekeeping-v1-AD-b → governance-v0)
- **CR comments seen:** 1 review summary + 3 inline + 1 issue-walkthrough
- **Actionable findings ingested:** 4 (1 outside-diff Major + 3 inline)
- **New findings this poll:** 4 (first poll on this PR)
- **Findings addressed since last poll:** 0 (n/a — first poll)
- **Counters:** critical 0/0/0 | major 2/0/0 | medium 0/0/0 | low 2/0/0 | nit 0/0/0
- **Recommendation:** request-changes (2 majors open in fix-in-pr)
- **YAML:** .claude/PRPs/reviews/pr-79-findings.yaml
- **Notes:** PR has been open since 2026-04-20 (3 days); CR review landed within 2 min of open. Issue-comment is CR walkthrough+pre-merge-check, not an actionable finding (1 'Description check' warning about template, not a code finding).

---

## bm: push — 2026-04-23T02:07:51Z
- **branch:** phase-v1-AD-d
- **commits pushed:** 3 (cb5a245ef..e7a2ba85c)
  - cb5a245ef docs(decision-queue): log v1-AD-d retro items as DQ #42-#46
  - 9e61d8c36 test(admin-audit-stream): add live SSE emission e2e test (task 6b)
  - e7a2ba85c docs(decision-queue): resolve DQ #45 (test-substitution policy) + sync v1-AD-d reports
- **remote ref:** origin/phase-v1-AD-d @ e7a2ba85c (fast-forward from bf5eb1a3b)
- **upstream tracking:** newly set (branch was not tracking before this push; remote ref existed from earlier impl-session push of bf5eb1a3b)
- **context:** incremental push following advisor-review follow-up on v1-AD-d Deviation 2. Last two commits (9e61d8c36, e7a2ba85c) landed as DQ #45 resolution + the live-SSE emission test that addresses the `live_yielded=false` substitution-policy finding.
- **PR status:** no PR open for this branch — next step is /bm-pr.
- **next:** /bm-pr

---

## bm: PR opened — 2026-04-23T02:11:00Z
- **PR:** #87 — Phase v1-AD-d — Dashboard aggregate + SSE audit stream
- **URL:** https://github.com/barrie-cork/lemmy/pull/87
- **Base ← Head:** governance-v0 ← phase-v1-AD-d
- **Body source:** completion-report + retro + plan + 11-commit log (governance-v0..HEAD)
- **Draft?** No (CR-eligible)
- **Title derivation:** branch pattern `phase-v1-AD-d` → `Phase v1-AD-d — <plan H1 tail>`; plan H1 is `Plan: v1-AD-d — Dashboard aggregate + SSE audit stream`.
- **Deviation callout:** PR body highlights Deviation 2 (test-substitution reversed on advisor review, task 6b added the live-SSE emission test) in a dedicated section so CR + human reviewers see it before scanning the diff.
- **Orthogonal commit flagged:** `bf5eb1a3b` (BM subagent + dispatchers) called out in body as tooling-on-phase-branch, so reviewers skip it under feature-diff review.
- **Closes:** none. v1-AD-c chores #82–#85 remain open under separate follow-up scope.
- **Next:** wait ~5–10 min for CodeRabbit; then `/bm-poll-cr 87`. Optional `/bm-ping pr-ready` (asks first).

---

## bm: poll-cr — 2026-04-23T09:04:41Z
- **PR:** #87 (phase-v1-AD-d → governance-v0)
- **head SHA:** e7a2ba85c (first poll, no prior head to compare)
- **PR age at poll:** 7.2 min (opened 08:56:48Z)
- **CR comments seen:** 1 (0 review / 0 inline / 1 issue)
  - Single issue comment is CR's "review in progress" placeholder (id 4303076441, posted 08:57:01Z) — no severity header, not an actionable finding per Phase 3 parse rules.
- **Actionable findings ingested:** 0 (0 from walkthrough/pre-merge — walkthrough not posted yet)
- **New findings this poll:** 0
- **Findings addressed since last poll:** 0 (n/a — first poll)
- **Counters:** critical 0/0/0 | major 0/0/0 | medium 0/0/0 | low 0/0/0 | nit 0/0/0
- **Recommendation:** pending (CR still processing; no findings to triage yet)
- **YAML:** .claude/PRPs/reviews/pr-87-findings.yaml (created, 1336 bytes)
- **Notes:** Expected zero-findings state for a PR this young. CR placeholder confirms bot is alive and processing (Run ID e5ac7e92-c8a5-41ce-9a45-25dfee4a3b96, ASSERTIVE profile, Pro plan, reviewing 30 files). Did NOT emit the "zero findings on a PR open >30 min" warning since PR age < 10 min per the context-documented carve-out. Next /loop poll at ~09:13 UTC (CronCreate job 1c243a4d, */10 * * * *) should see the real walkthrough + inline findings.
- **Next:** wait for next automated poll at ~09:13 UTC; or run `/bm-poll-cr 87` manually once the "review in progress" comment flips to a walkthrough.

---

## bm: poll-cr — 2026-04-23T09:19:59Z
- **PR:** #87 (phase-v1-AD-d → governance-v0)
- **head SHA:** e7a2ba85c (unchanged since poll #1 — no impl commits or force-pushes)
- **PR age at poll:** ~23 min (opened 08:56:48Z)
- **CR comments seen:** 21 (1 review / 19 inline / 1 issue)
  - Review summary (id 4161125651, submitted 09:08:35Z) reports "Actionable comments posted: 19" + 1 outside-diff-range finding in walkthrough.
  - Issue-walkthrough posted at 09:08:32Z (CR's full review landed ~11 min after PR open).
- **Phase 5.1 decision:** head unchanged BUT 20 new CR comments → PROCEED with merge-write ("No head / Yes CR" row).
- **Actionable findings ingested:** 20 (19 inline + 1 outside-diff from review summary)
- **New findings this poll:** 20 (cr-1 through cr-20, all new IDs since poll #1 had empty findings[])
- **Findings addressed since last poll:** 0 (no commits since poll #1; head SHA unchanged)
- **Counters:** critical 0/0/0 | major 16/0/0 | medium 0/0/0 | low 3/0/0 | nit 1/0/0
- **Recommendation:** request-changes (16 majors open in fix-in-pr; no critical)
- **YAML:** .claude/PRPs/reviews/pr-87-findings.yaml (9770 bytes)
- **Feature-vs-tooling split (for triage context):**
  - **9 findings on v1-AD-d feature surface** (cr-12–cr-20): governance DTOs, admin_dashboard, admin_audit_stream (SSE retry field, unbounded channel), audit_projection (scrub/redaction), admin_reputation_stats (SQL-helper widening, outside-diff), e2e.rs (admin-gate error matching, per-admin cap test isolation).
  - **11 findings on BM tooling** (cr-1–cr-11): .claude/commands/bm/*.md + .claude/rules/branch-manager.md. Introduced in bf5eb1a3b — orthogonal to v1-AD-d per PR body. Candidates for `carry-forward` at triage time (filed as follow-up chore issue) rather than `fix-in-pr` on this PR.
- **CR profile used:** ASSERTIVE, Plan: Pro, Run ID e5ac7e92 (reviewed 30 files).
- **Noteworthy findings (highest-signal):**
  - cr-2, cr-3 (Major): bm-merge.md spec holes — fix-in-pr gate asks for schema-invalid rows; PENDING checks allow race.
  - cr-5, cr-6 (Major): bm-poll-cr.md spec holes — walkthrough findings collide on (source, cr_url); head-SHA+count short-circuit misses edits/deletes.
  - cr-14 (Major): SSE driver uses unbounded mpsc — governance event flood can balloon memory.
  - cr-15 (Major): SSE `retry:` field emitted as `event: retry\ndata: 10000` — browsers won't respect reconnection delay.
  - cr-16 (Major): admin_dashboard per-community ruleset query silently truncated at LIMIT 100 — no has_more flag.
  - cr-17 (Major): audit_projection returns reason/denial_reason without scrub() — PII leak surface on admin dashboard.
  - cr-19 (Major): e2e per-admin-cap test relies on unique usernames but the cap is process-global; test can pass under contention that real deployments would hit.
  - cr-20 (Major, outside-diff): admin_reputation_stats.bucket_query widens `column: &str` as pub(crate) — SQL-injection footgun for next caller.
- **Cache files kept** at .claude/PRPs/reviews/.cr-cache/ for diagnosis (safe — gitignored).
- **Next:** `/bm-triage 87` to bucket findings (11 BM-tooling → carry-forward as separate chore issue; 9 feature-surface → fix-in-pr for impl session). Optionally `/bm-ping cr-posted` first (asks before sending Telegram).

---

## bm: triage — 2026-04-23T09:31:00Z
- **PR:** #87 (phase-v1-AD-d → governance-v0)
- **Buckets after triage:** fix-in-pr 4 | rebut 5 | carry-forward 11 | done 0 | wont-fix 0
  - fix-in-pr: cr-13 (nit, CommunityId newtype), cr-14 (major, bounded mpsc), cr-15 (major, SSE retry: field + e2e test update same commit), cr-18 (low, assert specific variant)
  - rebut: cr-12 (PRD §6.2 widget matrix), cr-16 (plan §4.1 line 111 bounded-at-100), cr-17 (ADR-015 + governance_log.rs:59 scrub_json write-time), cr-19 (plan §3 line 107 + §10 task 5 GOTCHA line 1305 process-global best-effort accepted), cr-20 (plan §10 task 3 GOTCHA line 1245 deliberate pub(crate); only bind is Option<i32> community_bind)
  - carry-forward: cr-1..cr-11 (all BM tooling findings — consolidating into ONE chore(bm) issue per feedback_pr_review_triage_pattern.md)
- **Comment drafted:** `.claude/PRPs/reviews/pr-87-comment.md` (READY TO POST — subagent context cannot invoke AskUserQuestion; handed back to parent for confirmation)
- **Carry-forward issues filed:** 0 (ASK gate awaits parent session; plan is ONE consolidated chore(bm) issue)
- **Recommendation:** request-changes (2 majors in fix-in-pr: cr-14 unbounded mpsc, cr-15 SSE retry-field spec violation)
- **YAML state:** counters + recommendation regenerated; rationales on all 5 rebuts cite plan/PRD/ADR sources
- **Next:** parent session runs AskUserQuestion for (a) post digest comment, (b) file ONE consolidated chore(bm) carry-forward issue. Then impl session addresses 4 fix-in-pr findings.

---

## bm: triage-outbound — 2026-04-23T09:41:00Z
- **PR:** #87
- **Digest comment posted:** https://github.com/barrie-cork/lemmy/pull/87#issuecomment-4303383461
- **Carry-forward issue filed:** https://github.com/barrie-cork/lemmy/issues/88 — "chore(bm): address CR findings cr-1..cr-11 on branch-manager tooling (from PR #87)"
- **Labels created on repo:** `carry-forward` (color fbca04), `source-coderabbit` (color d4c5f9)
- **YAML updated:** all 11 carry-forward findings' `notes:` now carry `filed: <issue-url>`
- **User approval:** AskUserQuestion single-batch confirm on both gates (post digest + file issue); both "as-is (Recommended)"
- **Next:** impl session addresses 4 fix-in-pr (cr-13 CommunityId newtype + cr-14 bounded mpsc + cr-15 retry: field + e2e test update + cr-18 specific variant assert). Then `/bm:bm-poll-cr 87` to flip fix-in-pr → done with addressed_in SHAs. Then `/bm:bm-merge 87` (asks before merging).

---

## bm: push — 2026-04-23T10:45:00Z
- **branch:** phase-v1-AD-d
- **commits pushed:** 3 (0699a1ac0..546348236)
  - 0699a1ac0 fix(admin-audit-stream): bounded SSE channel + retry: field (cr-14, cr-15)
  - 6c1654cb7 test(admin-gate): tighten assertions + CommunityId newtype (cr-13, cr-18)
  - 546348236 fix(admin-audit-stream): hoist SSE_CHANNEL_CAPACITY to module scope (clippy)
- **remote ref:** origin/phase-v1-AD-d @ 546348236 (fast-forward from e7a2ba85c)
- **upstream tracking:** already set from prior push; `-u` re-asserts
- **context:** v1-AD-d fix session follow-up — impl addressed all 4 fix-in-pr CR findings from triage (cr-13, cr-14, cr-15, cr-18). Local validation passed: cargo check --workspace --features full (0), cargo test --test e2e --no-run -p lemmy_server (0), cargo clippy --workspace --features full --no-deps -D warnings (0), e2e admin_dashboard + admin_audit_stream subsets green, Docker preflight OK.
- **PR status:** PR #87 already open; GitHub picked up push (headRefOid now 546348236). CR will re-review on push; findings cr-13, cr-14, cr-15, cr-18 expected to flip `fix-in-pr` → `done` with `addressed_in` SHAs on next poll.
- **next:** `/bm-poll-cr 87` in ~5–10 min (CR re-review on push).

---

## bm: poll-cr — 2026-04-23T12:16:48Z
- **PR:** #87 (phase-v1-AD-d → governance-v0)
- **head SHA:** 546348236 (advanced from e7a2ba85c since poll #2)
- **CR comments seen:** 2 review summaries + 21 inline + 1 issue-walkthrough
- **Actionable findings ingested:** 22 total (20 pre-existing cr-1..cr-20 + 2 new cr-21, cr-22)
- **New findings this poll:** 2
  - cr-21 (low) admin_audit_stream.rs:10 — module doc accuracy (trigger fires on signature NULL→NOT NULL transition, not every insert)
  - cr-22 (major) admin_dashboard.rs:130 — filter `signature.is_not_null()` on `recent_config_changes` query to exclude unsigned rows
- **Findings addressed since last poll:** 4 (cr-13 @ 6c1654cb7, cr-14 @ 0699a1ac0, cr-15 @ 0699a1ac0, cr-18 @ 6c1654cb7) — bucket remains `fix-in-pr`; /bm-triage promotes to `done`
- **Counters:** critical 0/0/0/0/0 | major 3/0/5/9/0 | medium 0/0/0/0/0 | low 2/0/0/2/0 | nit 1/0/0/0/0 (open/done/rebutted/carry_forward/wont_fix)
- **Recommendation:** request-changes (3 majors in fix-in-pr — cr-14/cr-15 have addressed_in but bucket not yet promoted to done; cr-22 new, unaddressed)
- **YAML:** .claude/PRPs/reviews/pr-87-findings.yaml (14860 bytes)
- **Notes:** 1 CR "Duplicate comment" on admin_audit_stream.rs 141-156 (filter non-admin notifications before bounded queue) — follow-up to cr-14 fix, same file same-class issue but not ingested as a separate finding per Phase 4 URL-key dedupe (CR flagged it under the duplicate-comments banner, no new discussion anchor). Walkthrough-related; if impl wants it addressed, raise as new cr-N on next push. Otherwise /bm-triage should promote cr-13/14/15/18 → done (addressed_in SHAs present) and bucket cr-21, cr-22 as fix-in-pr for impl follow-up.

---

## bm: triage — 2026-04-23T12:22:00Z (run #2)
- **PR:** #87 (phase-v1-AD-d → governance-v0)
- **Head SHA at triage:** 5463482367b33f3c844422ed34fd481378e6b71f (matches poll #3 last_polled_head_sha 546348236)
- **Bucket transitions this run:**
  - cr-13 (nit) fix-in-pr → done (addressed_in 6c1654cb7 verified)
  - cr-14 (major) fix-in-pr → done (addressed_in 0699a1ac0 verified)
  - cr-15 (major) fix-in-pr → done (addressed_in 0699a1ac0 verified)
  - cr-18 (low) fix-in-pr → done (addressed_in 6c1654cb7 verified)
  - cr-21 (low, new from poll #3) → fix-in-pr (doc-fix, impl to land pre-merge)
  - cr-22 (major, new from poll #3) → fix-in-pr (real bug: signature.is_not_null() filter on recent_config_changes)
- **Unchanged from triage #1:** 5 rebut (cr-12, cr-16, cr-17, cr-19, cr-20) + 11 carry-forward (cr-1..cr-11, all ref issue #88); CR re-review did not post follow-ups on any of them.
- **Buckets after triage #2:** fix-in-pr 2 | rebut 5 | carry-forward 11 | done 4 | wont-fix 0 (total 22)
- **Severity breakdown:** critical 0/0/0/0/0 | major 1/2/5/9/0 | medium 0 | low 1/1/0/2/0 | nit 0/1/0/0/0 (open/done/rebutted/carry_forward/wont_fix)
- **Carry-forward issues filed this run:** 0 (all 11 existing rows already reference issue #88 from triage #1; no new carry-forward findings this round)
- **Comment posted?** awaiting-confirmation — user asked via AskUserQuestion before `gh pr comment 87 --repo barrie-cork/lemmy --body-file .claude/PRPs/reviews/pr-87-comment.md`. Draft updates previous digest #4303383461.
- **Recommendation:** request-changes (1 major + 1 low still in fix-in-pr)
- **YAML:** .claude/PRPs/reviews/pr-87-findings.yaml (counters regenerated)
- **Next:** impl session to land cr-21 + cr-22 in a single docs+fix commit → push → `/bm-poll-cr 87` for poll #4 → `/bm-triage 87` for run #3 → if all green, `/bm-merge 87`.

---

## bm: push — 2026-04-23T13:10:00Z
- **branch:** phase-v1-AD-d
- **commits pushed:** 1 (546348236..8b99a3401)
  - 8b99a3401 fix(admin-dashboard): filter unsigned rows + correct SSE doc (cr-21, cr-22)
- **remote ref:** origin/phase-v1-AD-d @ 8b99a3401 (fast-forward from 546348236)
- **upstream tracking:** already set (from earlier push this session)
- **context:** impl-side fix for CR poll #3 findings cr-21 (low, doc accuracy) + cr-22 (major, unsigned-row filter bug). Validation complete in impl session: cargo check/test/clippy all green, 4 admin_dashboard e2e tests passed (including new `excludes_unsigned_rows` test), 3 admin_audit_stream e2e tests passed, Docker preflight OK.
- **PR status:** #87 open, awaiting CR re-review on new push. Previous triage-digest (.claude/PRPs/reviews/pr-87-comment.md) is stale w.r.t. cr-21/cr-22 — user elected to defer fresh digest until poll #4 + triage #3.
- **Next:** `/bm-poll-cr 87` in 5-10 min to ingest CR's re-review of 8b99a3401.

---

## bm: poll-cr — 2026-04-23T14:25:37Z
- **PR:** #87 (phase-v1-AD-d → governance-v0)
- **head SHA:** 8b99a3401 (changed since last poll — advanced from 546348236)
- **CR comments seen:** 3 reviews / 23 inline / 1 issue (walkthrough)
- **Actionable findings ingested:** 24 (0 new from walkthrough/pre-merge this poll)
- **New findings this poll:** 2 (cr-23 admin_dashboard.rs:81 Major refactor | cr-24 admin_dashboard.rs:218 Major perf)
- **Findings addressed since last poll:** 2 (cr-21, cr-22 → `addressed_in: 8b99a3401`; bucket stays `fix-in-pr` until triage promotes)
- **Counters:** critical 0/0/0/0/0 | major 3/2/5/9/0 | medium 0 | low 1/1/0/2/0 | nit 0/1/0/0/0 (open/done/rebutted/carry_forward/wont_fix)
- **Recommendation:** request-changes (3 major fix-in-pr: cr-22 with addressed_in pending triage-promote, plus cr-23 + cr-24 new)
- **YAML:** .claude/PRPs/reviews/pr-87-findings.yaml (24 findings, counters regenerated)
- **Notes:** CR review #4163153765 (submitted 14:08:47Z) re-reviewed diff 5463482367..8b99a3401, posted 2 actionable inline comments on admin_dashboard.rs (both Major). Also contains a "Duplicate comments" block re-raising admin_audit_stream kind-filter-before-try_send (enhancement to cr-14 bounded-channel fix) — NOT ingested as a separate finding because CR tagged it Duplicate and it has no stable inline cr_url anchor (review-body-only). cr-23 invokes ADR-013 coding guideline for exhaustive CaseStatus match. cr-24 is a perf/pool-pressure finding (N+1 queries in per-community loop). Neither finding was on the impl's expected CR-comment list.
- **Next:** `/bm-triage 87` for run #3 to (a) promote cr-21/cr-22 fix-in-pr → done, (b) triage cr-23 + cr-24 (both look like legit fix-in-pr candidates on first read; cr-23 is ADR-013 compliance pressure so impl may want to address in this PR, cr-24 is perf-pressure that could plausibly go carry-forward for v2). User decides disposition.

---

## bm: push — 2026-04-23T15:01:35Z
- **branch:** phase-v1-AD-d
- **commits pushed:** 1 (8b99a3401..7ea0844cd)
  - 7ea0844cd refactor(admin-dashboard): exhaustive CaseStatus + batched cfg query (cr-23, cr-24)
- **remote ref:** origin/phase-v1-AD-d @ 7ea0844cd (fast-forward from 8b99a3401)
- **upstream tracking:** already set
- **context:** PR #87 fix-in-pr push addressing cr-23 (ADR-013 exhaustive CaseStatus match via `is_active_status(CaseStatus)` + typed Diesel `group_by/count_star` replacing raw-string filter) and cr-24 (N+1 per-community `get_int_opt` loop replaced by single batched SELECT against `governance_config_current` keyed on `scope IN (...) OR scope='instance'`; map-lookup replays Community→Instance cascade semantics). `rule_sets_summary` no longer takes cache or pool. New e2e test `admin_dashboard_per_community_active_version_cascade` verifies 42 (community-scoped wins) vs 999 (instance-scoped fallback). User elected both in-PR under auto mode.
- **validation:** cargo check --workspace --features full → 0; cargo test --test e2e --no-run -p lemmy_server → 0; cargo clippy --workspace --features full --no-deps -- -D warnings → 0; e2e admin_dashboard × 5 pass; e2e admin_audit_stream × 3 pass.
- **PR status:** PR #87 OPEN, headRefOid=7ea0844cd confirmed; CR will re-review on push.
- **next:** `/bm-poll-cr 87` in 5–10 min (then `/bm-triage 87` → post digest → `/bm-merge 87`).

---

## bm: poll-cr — 2026-04-23T15:15:00Z
- **PR:** #87 (phase-v1-AD-d → governance-v0)
- **head SHA:** 7ea0844cd (changed since last poll — advanced from 8b99a3401)
- **CR comments seen:** 4 reviews / 24 inline / 1 issue (walkthrough)
- **Actionable findings ingested:** 25 (0 new from walkthrough/pre-merge this poll)
- **New findings this poll:** 1 (cr-25 e2e.rs:6024 Minor → low — tighten `>= 1` to `Some(1)` exact assertions for status counts)
- **Findings addressed since last poll:** 2 (cr-23, cr-24 → `addressed_in: 7ea0844cd` via commit-subject heuristic; bucket stays `fix-in-pr` until triage promotes)
- **Counters:** critical 0/0/0/0/0 | major 3/2/5/9/0 | medium 0 | low 2/1/0/2/0 | nit 0/1/0/0/0 (open/done/rebutted/carry_forward/wont_fix)
- **Recommendation:** request-changes (3 major fix-in-pr: cr-22/cr-23/cr-24 all with `addressed_in` set, pending triage-promote; + 2 low: cr-21 addressed_in=8b99a3401 pending promote, cr-25 new/unaddressed)
- **YAML:** .claude/PRPs/reviews/pr-87-findings.yaml (25 findings, counters regenerated)
- **Notes:** CR review #4163587247 (submitted 15:07:35Z) re-reviewed diff 8b99a3401..7ea0844cd and posted exactly 1 actionable finding: cr-25 at `crates/server/tests/e2e.rs:6024` (Minor, tighten `>= 1` assertions to `Some(1)` exact — CR flags that current loose comparison would mask double-counting regressions in `count_active_cases`). No follow-ups on cr-23/cr-24 (CR considers those addressed by the refactor). No comments on the `is_active_status` / `status_key` helpers, the `LIMIT 100` guardrail, or the cascade test — all considered acceptable. No "Duplicate comments" block this time. Commit subject `refactor(admin-dashboard): exhaustive CaseStatus + batched cfg query (cr-23, cr-24)` matched heuristic cleanly; both inline findings mapped to `addressed_in: 7ea0844cd`.
- **Next:** `/bm-triage 87` for run #4 to (a) promote cr-21 + cr-22 fix-in-pr → done (addressed_in=8b99a3401), (b) promote cr-23 + cr-24 fix-in-pr → done (addressed_in=7ea0844cd), (c) triage cr-25 (low/test-quality nit — plausibly fix-in-pr quick-patch or carry-forward depending on appetite). After triage, post fresh digest then `/bm-merge 87`.
