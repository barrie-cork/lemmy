---
phase: brehon-conformance-audit
role: bm-task
task: bm-triage
brief_n: 1
authored: 2026-05-21
canonical_ref: .claude/PRPs/briefs/federation-inbound-a-bm-triage.md
---

# [role:bm-task] brehon-conformance-audit bm-triage — draft four-bucket triage on PR #141 — see .claude/PRPs/briefs/brehon-conformance-audit-bm-triage.md

## §1 Role + dispatch

`[role:bm-task] brehon-conformance-audit bm-triage — draft four-bucket triage on PR #141 (23 findings: 0 critical + 8 major + 0 medium + 14 low + 1 nit; all CR)`

## §2 Scope

Run `bm-triage` for PR #141 (`phase-brehon-conformance-audit` → `governance-v0`).

Findings YAML at `.claude/PRPs/reviews/pr-141-findings.yaml` has
**23 findings** (all `source: coderabbit`; no Copilot — only CR ran
on this PR):

- **0 critical**, **8 major**, **0 medium**, **14 low**, **1 nit**.
- Currently every finding has `bucket: fix-in-pr` placeholder from
  poll-cr — **re-triage every row**; do NOT trust the placeholder.

**The 8 majors** (one-line summaries — fetch full text via `gh api`):

| ID | File:line (approx) | Summary |
|---|---|---|
| `cr-1` | `.claude/decision-queue.json:~1860` | Require explicit user-approval metadata for DQ #307 (advisor) + #311 (planner) — judgment-heavy resolutions need `approved_by` + `approved_at` fields |
| `cr-6` | (verify path via `cr_url`) | "Resolve the cargo-check requirement contradiction" between two parts of plan/brief/skill |
| `cr-13` | `.claude/PRPs/plans/brehon-conformance-audit.plan.md` Task 8a section | "Correct Task 8a clippy expectation text" — the prior expectation text needs to match the mechanism-revision reality |
| `cr-16` | `.claude/skills/brehon-conformance-audit/audit-metrics.schema.json` or `METRICS.md` | "Constrain metrics row objects to prevent silent malformed data" — schema-strictness gap |
| `cr-18` | `audit-metrics.schema.json` or `compute-metrics.sh` | "Field names are inconsistent with the metrics payload contract" — schema↔script field-name drift |
| `cr-19` | `.claude/skills/brehon-conformance-audit/METRICS.md` or `compute-metrics.sh` | "False-positive attribution is implemented incorrectly" — FP attribution rule defect |
| `cr-20` | `compute-metrics.sh` or `METRICS.md` | "Lead-time omits compile-caught events" — formula bug undercounts lead-time |
| `cr-21` | `.claude/skills/brehon-conformance-audit/scripts/find-sibling.sh` | "Rust function signature matcher is too narrow and misses valid top-level functions" — grep too restrictive |

**The 14 low + 1 nit** are typically prose/markdown/style nits OR
incremental schema/metrics suggestions. Judge per the criteria below.

**Critical lens for this triage** — this phase delivers a brehon
**skill bundle** (not Rust crate code). The skill is markdown + bash +
Python + JSON-schema + clippy.toml. Findings split into roughly:

1. **Skill-correctness findings** (`cr-16/18/19/20/21` family) — `audit-
   metrics.schema.json`, `compute-metrics.sh`, `METRICS.md`, `find-
   sibling.sh`. These ARE the skill product. If CR has spotted a real
   defect → `fix-in-pr`. If CR's suggestion is a refinement to a
   shipped-and-working calibration → `carry-forward` (the dogfood test
   showed precision/recall 1.000 + 27.5h lead-time on the only sample
   we have; over-refining without more samples may hurt calibration).
2. **Plan/brief/runlog prose findings** (`cr-13` family) — advisor
   meta-docs. Default `wont-fix` or `carry-forward` per brief-file rule
   below.
3. **DQ-schema findings** (`cr-1` family) — `decision-queue.json`. This
   is a schema-evolution proposal (add `approved_by` / `approved_at`
   fields). Treat as `carry-forward` to a DQ-schema-v3 follow-up sub-
   phase; the current `kind: log` + `answered_by` audit trail is the
   ratified pattern (per `feedback_pmd_two_memory_systems_distinction`
   + 2026-05-20 user direction at User Gate 2). Do NOT auto-`fix-in-pr`
   a schema change.
4. **Cargo-check contradiction (`cr-6`)** — read full `cr_url` text;
   bucket on whether the contradiction is a real defect (skill says X
   in one place + Y in another) → `fix-in-pr` with one-line clarif; or
   CR misread (e.g. one section was talking about workspace-check
   while another was talking about per-module deny) → `rebut`.

Draft `.claude/PRPs/reviews/pr-141-comment.md` summarising the triage
for user review **before** posting.

**Do NOT post the PR comment.** User must approve via USER-GATE-3 first.

## §3 Required reading

- `.claude/commands/bm/bm-triage.md` — bm-triage verb (Phase 1-8
  operational script)
- `.claude/refs/pr-review-triage.md` — four-bucket triage pattern
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (bucket /
  rationale / addressed_in invariants)
- `.claude/lessons/feedback_pr_review_triage_pattern.md` — bucketing
  heuristics
- `.claude/lessons/feedback_severity_labels_dont_imply_semantic.md` —
  CR-tagged severity is a hypothesis; the "if I revert, does the
  symptom return" test is authoritative
- `.claude/lessons/feedback_verify_automated_reviewer_claims_against_compiler.md`
  — CR claims about Rust code are hypotheses; verify against the
  compile + the user-ratified DQ history before bucketing fix-in-pr
- `.claude/lessons/feedback_coderabbit_block_merge_critical.md` — CR
  Critical normally blocks merge (n/a here — 0 critical on this PR)
- `.claude/PRPs/reports/brehon-conformance-audit-retro.md` — the
  shipped retro at `425ab13c8` documents the cycle-3 catch-fire +
  mechanism revision; many CR findings may relitigate decisions
  already ratified there
- Each finding's `cr_url` — read the full review text via `gh api
  repos/barrie-cork/lemmy/pulls/comments/<id>` when the summary alone
  is insufficient to bucket

## §4 Constraints

- **--repo barrie-cork/lemmy** on all `gh` commands.
- Every finding gets exactly one bucket: `fix-in-pr` / `rebut` /
  `carry-forward` / `done` / `wont-fix`. Never empty, never invented.
- Re-triage **every** finding — the placeholder `bucket: fix-in-pr`
  set by poll-cr is NOT a triage decision.
- Write `rationale` for each finding (1-2 sentences: what bucket + why).
- **Brief / plan / runlog / DQ findings** (findings whose `file` is
  under `.claude/PRPs/briefs/**`, `.claude/PRPs/plans/**`,
  `.claude/PRPs/reports/**`, `.claude/runlog/**`, or
  `.claude/decision-queue.json`) — advisor meta-artifacts, NOT shipped
  code. Default bucket `wont-fix` or `carry-forward` with rationale
  "advisor meta-doc; CR prose-quality finding, not a code defect"
  UNLESS the finding identifies a factual error that would mislead a
  future reader (then `fix-in-pr` IF the artifact is on this PR's diff
  AND the fix is small).
- **Skill-content findings** (files under `.claude/skills/brehon-
  conformance-audit/**`) — these ARE the shipped product. Judge on
  correctness:
  - Real defect (calibration breaks; metric formula wrong;
    schema-script drift) → `fix-in-pr`.
  - Refinement / additional safety (already-correct code could be more
    defensive) → `carry-forward` to a `brehon-conformance-audit-r1`
    follow-up.
  - Mis-read (CR didn't understand the dogfood evidence) → `rebut`.
- **DQ-schema findings (`cr-1`)** — schema evolution proposals. Always
  `carry-forward` to a DQ-schema-v3 follow-up sub-phase unless user
  overrides at User Gate 3. Rationale: "DQ schema evolution is a
  cross-cutting concern; current schema-v2 has zero blocking
  defects (51 entries live, all readable). User-approval metadata is
  reasonable but needs its own design pass + ADR + retrofit migration."
- **Cargo-check contradiction (`cr-6`)** — read full `cr_url` body.
  If real contradiction → `fix-in-pr` with the smaller scope (likely
  a one-line clarify in SKILL.md or METRICS.md). If misread →
  `rebut` citing the two sections in context.
- For `bucket: fix-in-pr`: leave `addressed_in: null` (impl fills on
  fix commit).
- For `bucket: rebut`: rationale must explain precisely why CR was
  wrong (cite the compiling code / the user-ratified retro / the
  dogfood evidence). User gate decides whether to post the rebuttal.
- For `bucket: done`: rationale cites the existing commit SHA that
  already addresses it.
- For `bucket: carry-forward`: **file the GH issue** via `gh issue
  create --repo barrie-cork/lemmy` (title: `[brehon-conformance-audit-
  r1] <one-line>`, body: cite this PR + finding id + cr_url + the
  one-line summary), capture the issue URL into `notes:`. **ASK USER
  via AskUserQuestion before creating each carry-forward issue** per
  branch-manager.md autonomy rules.
- Group findings that share a root cause (note the grouping in each
  member's rationale; recommend a single shared fix commit).
- Regenerate the `counters` block from `findings[]` (open/done/
  rebutted/carry_forward/wont_fix per severity).
- Recompute top-level `recommendation` per `bm-prp-review.md` Phase 6
  condition table:
  - Any `bucket: fix-in-pr` with `addressed_in: null` AND severity in
    `{critical, major}` → `recommendation: block`.
  - Else if any `fix-in-pr` major addressed but unverified →
    `request-changes`.
  - Else → `approve`.
- Write the draft PR comment at `.claude/PRPs/reviews/pr-141-comment.md`
  (markdown, group by bucket, ~1-line per finding, lead with the
  recommendation + bucket counters table).
- **Do NOT post the PR comment** (USER-GATE-3 decides).
- **Do NOT push** the findings YAML or comment draft until user
  approval. Both files are gitignored per `.gitignore` patterns
  (`.claude/PRPs/reviews/pr-*-findings.yaml` + `pr-*-comment.md`);
  verify with `git check-ignore -q <path>; echo $?` (exit 0 = ignored).
  Force-add + commit is **the bm-merge POST-gate step**; for triage,
  leave both files in the worktree and report paths in task output.
  (The advisor reads them from the daemon worktree for USER-GATE-3.)
- **Worker branch must be on `phase-brehon-conformance-audit`** —
  base_branch=phase-brehon-conformance-audit (NOT governance-v0 like
  the prior poll-cr brief — that lesson cost us a cherry-pick recovery
  per session post-mortem 2026-05-21). The findings YAML lives on the
  phase branch so it merges with PR #141; the triage updates the YAML
  in place, so the worker must be on the same branch as the file.
- Do NOT edit `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**` (BM file-ownership boundary).
- Report in task output: per-bucket counts (fix-in-pr / rebut /
  carry-forward / done / wont-fix), the 8 majors' bucket assignments
  + one-line rationale each, recommendation (block / request-changes /
  approve), and both artifact paths.

## §5 Out of scope

- Posting the PR comment (USER-GATE-3).
- Any fix-in-PR code commit (post-gate impl-task work).
- Merging the PR (User Gate 5 + bm-merge).
- Editing the shipped skill content (under `.claude/skills/**`) —
  fix-in-pr findings get a separate impl-task brief post-gate.

## §6 Commit subject

The BM Junior's own commit (if applicable post-gate at the user's
direction) uses `chore(bm): triage #141 — <four-bucket counts>` per
the verb's Phase X template. For this brief's scope (draft only,
pre-gate), the BM SHOULD NOT commit — both artifacts are gitignored
and remain in the worker worktree for advisor pickup.
