---
phase: v1-federation-inbound-c
role: bm-task
task: bm-poll-cr
brief_n: 1
authored: 2026-05-22
---

# [role:bm-task] fed-in-c bm-poll-cr — ingest CodeRabbit findings on PR #144 — see .claude/PRPs/briefs/v1-federation-inbound-c-bm-poll-cr-1.md

## §1 Role + dispatch

`[role:bm-task] fed-in-c bm-poll-cr — ingest CodeRabbit findings on PR #144`

Actual create-task description (single line, <100 chars):

```
[role:bm-task] fed-in-c bm-poll-cr — see .claude/PRPs/briefs/v1-federation-inbound-c-bm-poll-cr-1.md
```

**Junior dispatch base_branch**: `phase-v1-federation-inbound-c` (NOT governance-v0). Brief committed on phase branch because (a) the canonical worktree had a concurrent-session UU at brief-author time blocking commit; (b) the BM bm-poll-cr work operates on PR #144 which targets the phase branch, so reading the brief from the phase branch's tree is operationally fine. Per advisor-orchestrator.md §2.1 the canonical convention is governance-v0, but the override is consistent with multi-lane-worktree.md "atomic protocol" hard refusal #6 — do not race a concurrent canonical-checkout session.

## §2 Scope

Run `bm-poll-cr` for PR #144 (`phase-v1-federation-inbound-c` → `governance-v0`).

CodeRabbit has posted **TWO** PR-level reviews on this PR:

- **CR review #1 @ 2026-05-21T23:43Z** — `**Actionable comments posted: 8**` — body ~6621 chars; findings primarily target `.claude/decision-queue.json` content (stale-state contradictions, narrative-vs-array-position drift, audit-trail rewrite concerns).
- **CR review #2 @ ~00:50Z (post-findings-shell-commit `89addc3d6`)** — `**Actionable comments posted: 1**` — body ~1688 chars; single finding on `pr-144-findings.yaml:6` (`opened_at` timestamp drift — recorded `2026-05-21T23:39:00Z` but PR was created at `2026-05-21T23:29:31Z`).

**Total: 9 actionable CR findings.** Ingest all 9 into the findings YAML plus any inline review comments. **Pre-existing shell at `.claude/PRPs/reviews/pr-144-findings.yaml`** (committed at phase tip `89addc3d6` by the advisor post-BM-#404-skip) — your job is to REPLACE the shell with the populated v1 of the YAML, preserving `pr`, `title`, `head`, `base` from the shell.

**PR:** `#144`
**Branch:** `phase-v1-federation-inbound-c`
**Repo:** `barrie-cork/lemmy`

Write `.claude/PRPs/reviews/pr-144-findings.yaml` per `.claude/PRPs/reviews/SCHEMA.md`. Force-add + commit + push it (gitignored by default — MUST be committed so the advisor can retrieve it for triage).

### §2a Source landscape (read before pulling — three+ review events on this PR)

`gh pr view 144` shows review/comment events. Disambiguate:

1. **`copilot-pull-request-reviewer` review @ 2026-05-21T23:35Z** (state COMMENTED, body_len 2674) — Copilot's PR overview. **OUT OF SCOPE for bm-poll-cr** (this verb ingests CodeRabbit only). Do NOT ingest Copilot comments as `source: coderabbit`. Note in the runlog that a Copilot review exists so the later `bm-triage` step can fold it in as `source: claude` if it has actionable content.
2. **`coderabbitai` review #1 @ ~23:43Z** (state COMMENTED, body_len 6621) — `**Actionable comments posted: 8**` — authoritative against pre-forward-merge tip `c51d00ce1`.
3. **`coderabbitai` review #2 @ ~00:50Z** (state COMMENTED, body_len 1688) — `**Actionable comments posted: 1**` — authoritative against post-findings-shell tip `89addc3d6` (this CR review fired because the PR diff changed when advisor force-added the findings.yaml shell + the forward-merge merge commit; CR re-reviewed the new diff).
4. **`coderabbitai` issue/walkthrough comment** — the auto-summary comment, NOT a findings-bearing review.

### §2b Head-SHA-vs-finding-timestamp skew (do NOT treat as a problem)

CR review #1 was posted against tip `c51d00ce1` (pre-forward-merge state). The current PR head is `89addc3d6` (post the advisor recovery: `da1e49683` forward-merge + `89addc3d6` findings.yaml shell). All 8 findings from review #1 still apply IF the underlying lines still exist on the current head — most target `.claude/decision-queue.json` content which has changed substantially via the v3 schema migration + union-by-id merge resolution. **Ground-truth EACH finding's `file:line` against `gh pr diff 144 --name-only` per Phase 1.5 as normal.** A finding whose line no longer exists on the current head should still be ingested but flagged in `notes:` as `[outside-diff post-merge — verify against current line if relevant]`. CR review #2 is fresh against the current tip and applies cleanly.

## §3 Required reading

- `.claude/commands/bm/bm-poll-cr.md` — bm-poll-cr operational script (follow phases 1→8 verbatim; note Phase 1.5 diff ground-truth, Phase 3 severity-token parse, Phase 4 stable `cr-<seq>` IDs, Phase 5.1 skip-write short-circuit, Phase 6 force-add+commit+push)
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (stable `id`, `bucket: ""` left blank for triage, `source`, `severity`, `summary`, `file:line`, `recommendation`, regenerated `counters`)
- `.claude/PRPs/reviews/pr-144-findings.yaml` (in-tree on phase branch at `89addc3d6`) — pre-existing shell to REPLACE (preserve `pr`, `title`, `head`, `base` fields; everything else gets repopulated)
- `.claude/rules/branch-manager.md` — BM autonomy bounds + file-ownership
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every gh command
- `.claude/lessons/feedback_python_utf8_encoding_windows.md` — YAML write MUST use `encoding="utf-8"` + `allow_unicode=True` (CR severity emoji)
- `.claude/PRPs/briefs/v1-AD-e-bm-poll-cr-1.md` — **canonical-sibling brief** (per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first); mirror its §1-§4 shape verbatim adjusting for two-review situation

## §4 Constraints

- **`--repo barrie-cork/lemmy`** on every `gh` command.
- Pull all 3 CR sources per command Phase 2 (one query each):
  - `repos/barrie-cork/lemmy/pulls/144/reviews` (PR-level — BOTH 8-findings and 1-findings reviews live here)
  - `repos/barrie-cork/lemmy/pulls/144/comments` (inline per-file/line)
  - `repos/barrie-cork/lemmy/issues/144/comments` (walkthrough / pre-merge check)
  - **Filter `select(.user.login == "coderabbitai[bot]")`** — this is what excludes the Copilot review automatically. Do NOT widen the filter.
- Parse severity from CR's second header token per the Phase 3 table (`Critical|Major|Medium|Minor|Nitpick` → `critical|major|medium|low|nit`). The emoji is decoration; the token is authoritative.
- Assign stable `cr-<seq>` IDs in chronological order across all CR sources. This is poll #1 so all are new (`cr-1`..`cr-9`).
- **`bucket: ""`** left blank on every finding — **do NOT bucket** (triage is the next, separate verb; bucketing here is a process breach).
- Regenerate `counters` from `findings[]`; set `last_poll_at`, `poll_count: 1`, `last_polled_head_sha: 89addc3d6` (OR newer if subsequent advisor commits land), `last_polled_fingerprint`.
- Preserve `pr`, `title`, `head`, `base` from the existing shell. Overwrite `findings: []` with the parsed findings array; overwrite `counters` with regenerated counts.
- Write `.claude/PRPs/reviews/pr-144-findings.yaml` with `encoding="utf-8"`, `allow_unicode=True`, `sort_keys=False`.
- **Force-add + commit + push** the YAML (per command Phase 6): `git add -f .claude/PRPs/reviews/pr-144-findings.yaml` → `git commit -m "chore(bm): poll-cr #144 — ingest CR findings (poll #1)"` → `git push origin phase-v1-federation-inbound-c`. (The advisor reads it off the phase branch for the triage gate.)
- Append the Phase 7 runlog entry to `.claude/runlog/bm-runlog.md` (`## bm: poll-cr — <ISO>` with counters + the §2a Copilot-review note + the §2b post-merge-head-skew note + the "two CR reviews, 8+1=9 total" note).
- Do **NOT** post any reply/comment on the PR (triage is separate; posting needs user confirmation per autonomy bounds).
- Do **NOT** send a Telegram ping (advisor handles outbound; no ping unless the user asks).
- Do **NOT** touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**` (BM file-ownership HARD boundary).
- **PRE-PUSH MANDATE (DQ #338 daemon-bug mitigation):** worker MUST pre-push the worker branch (`git push origin HEAD:$(git branch --show-current)`) after the findings.yaml + runlog commit, BEFORE the daemon's finalize step runs. Advisor will manually finalize-merge from `origin/junior/<worker-branch>` per the cohort-1 + Task-3 + Task-4 + bm-pr working template established this phase.
- If the parser finds **fewer than 9** actionable CR findings: do NOT silently proceed — emit a top-level `notes:` on the YAML + a runlog warning ("CR reviews said 8+1=9 actionable; parser ingested N") so the advisor can investigate before triage. CR's own count is the ground truth to reconcile against.

## §5 Validation gates

- Phase 1: `gh pr view 144` returns 200; head SHA captured (`89addc3d6` OR newer if subsequent advisor commits land).
- Phase 1.5: `gh pr diff 144 --name-only` produces the diff file list for ground-truthing.
- Phase 2: all 3 CR sources pulled; `coderabbitai[bot]` filter applied.
- Phase 3: severity tokens parsed; emoji discarded.
- Phase 4: 9 `cr-<seq>` ids assigned chronologically.
- Phase 5: counters regenerated; matches actual findings count.
- Phase 5.1: skip-write short-circuit DOES NOT fire (head SHA changed from null to `89addc3d6`; this is poll #1 against a populated YAML shell).
- Phase 6: force-add + commit `chore(bm): poll-cr #144 — ingest CR findings (poll #1)` + push.
- Phase 7: runlog append on phase-branch (trunk-side bm-runlog.md append is OPTIONAL for bm-poll-cr per canonical bm-poll-cr.md; advisor will handle trunk side if cross-phase visibility needed).
- Phase 8: return summary with counters + recommendation suggestion (e.g. "9 findings: M majors / N medium / N low / N nit; recommend full triage" — let advisor decide bucketing).

## §6 Expected output (return to advisor)

```
## bm-poll-cr complete — PR #144 findings ingested (poll #1)

**PR URL:** https://github.com/barrie-cork/lemmy/pull/144
**findings.yaml:** .claude/PRPs/reviews/pr-144-findings.yaml @ phase tip <new-sha>
**Counters:** total=9, by_source: coderabbit=9, claude=0, user=0
              by_severity: critical=N, major=N, medium=N, low=N, nit=N
              by_bucket: fix-in-pr=0, rebut=0, carry-forward=0, done=0, wont-fix=0 (all blank pre-triage)
**Copilot review noted in runlog:** yes (out of scope for bm-poll-cr; advisor/triage to fold in)
**Post-merge head-skew noted:** yes (CR review #1 against c51d00ce1; current head 89addc3d6; findings ground-truthed against current diff)
**Next suggested:** bm-triage to bucket the 9 findings into fix-in-pr/rebut/carry-forward/done/wont-fix
```

## §7 Why this brief differs from the canonical sibling

Mirrors `.claude/PRPs/briefs/v1-AD-e-bm-poll-cr-1.md` schema verbatim per advisor-orchestrator.md §3.6 canonical-schema-first gate. Differences:
- **Two CR reviews instead of one** (§2 + §2a explicitly call out the 8+1=9 split; PR #133 had a single 6-findings review).
- **Pre-existing findings.yaml shell to REPLACE** (advisor authored it post-BM-#404-skip at `89addc3d6`; preserve top-level metadata, repopulate findings[] + counters).
- **`opened_at` field IS one of the CR findings** (review #2 cr-9: timestamp drift — `2026-05-21T23:39:00Z` recorded vs PR-created at `2026-05-21T23:29:31Z`). Don't try to "fix" this in the shell as a pre-emptive action; just INGEST it as a finding (triage will decide whether to update or rebut).
- **PRE-PUSH MANDATE added (§4 last bullet)** — DQ #338 daemon-bug mitigation; fed-in-c's bm-pr brief had this pattern; same pattern applies here.
- **Brief committed on phase branch, not governance-v0** (§1 last paragraph): canonical worktree had a concurrent-session UU at author time; per multi-lane-worktree.md hard refusal #6 atomic protocol, do not race. Dispatching with `base_branch=phase-v1-federation-inbound-c` makes the brief visible to the BM Junior worker on its forked branch.
- **Two-runlog ordering note added (§5 Phase 7)** — phase-branch runlog append is BM-owned; trunk-side bm-runlog.md append for bm-poll-cr is OPTIONAL (the canonical bm-poll-cr.md script doesn't currently mandate it the way bm-pr does).

---

_Brief author: advisor session (lane-dedicated CWD `C:/Users/barri/Developer/brehon-fork-fed-in-c` on `phase-v1-federation-inbound-c`); brief committed on `phase-v1-federation-inbound-c` (override from canonical convention; rationale in §1 last paragraph). BM Junior #404 (bm-pr) skipped Phase 5 + Phase 6 trunk-runlog; advisor authored both as post-condition recovery (`feedback_bm_false_success_advisor_post_condition_catch.md` 4th confirmed occurrence)._
