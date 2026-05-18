---
phase: v1-AD-e
role: bm-task
task: bm-triage
brief_n: 1
authored: 2026-05-17
---

# [role:bm-task] AD-e bm-triage — draft four-bucket triage on PR #133 — see .claude/PRPs/briefs/v1-AD-e-bm-triage-1.md

## §1 Role + dispatch

`[role:bm-task] AD-e bm-triage — draft four-bucket triage on PR #133 (6 CR findings: 4 major + 1 low + 1 nit)`

## §2 Scope

Run `bm-triage` for PR #133 (`phase-v1-AD-e` → `governance-v0`).

The findings YAML is **already on the phase branch** at
`.claude/PRPs/reviews/pr-133-findings.yaml` (committed by the advisor's
cherry-pick of the bm-poll-cr worker output; phase tip `cf8c7bbdd`,
blob `4964cba5c`). **Do NOT re-run `bm-poll-cr`** — poll-cr is complete,
6 findings ingested and count-reconciled against CR's stated 6. Your job
is the four-bucket triage only.

### §2a CRITICAL — re-derive every bucket from scratch (do NOT trust the pre-set values)

The bm-poll-cr worker (a process miss it should not have made) wrote
`bucket: fix-in-pr` on **all 6 findings** even though its brief said to
leave `bucket: ""` blank. **Ignore those pre-set values entirely.**
Treat every finding as un-bucketed and classify it freshly using the
Phase-2 four-bucket table + the semantic-revert test
(`feedback_severity_labels_dont_imply_semantic.md`: "if I revert this
suggested change, does the symptom return?" — if no, it's likely
`rebut`/`wont-fix`, not `fix-in-pr`). Bucketing every finding
`fix-in-pr` by default is exactly the lazy classification the four-bucket
discipline exists to prevent. Each of the 6 gets an independent decision
with a written `rationale`.

### §2b The 6 findings + triage context

All 6 are `source: coderabbit`. **0 critical, 4 major, 1 low, 1 nit.**
None is a `source: claude` ADR-violation or cargo-failure row, so the
Phase-2 special-cases (auto-`fix-in-pr`) do NOT apply — every bucket is
a genuine judgment call.

| id | sev | file:line | summary | triage context (your call, not pre-decided) |
|---|---|---|---|---|
| cr-1 | low | `.claude/decision-queue.json:3962` | Fix timestamp chronology in newly added DQ entries | DQ #237/#238 — user-resolved at plan-approval (`7d8f84dfc`). Append-only audit ledger; historical timestamp idiosyncrasies are **forward-only, not rewritten** per `.claude/rules/decision-queue.md` ("Do not rewrite historical entries"). Weigh: is a chronology nit on an immutable audit row a real defect, or CR-noise on a file whose schema explicitly forbids back-editing? Likely `rebut` or `wont-fix` with that citation — but read the actual diff hunk (`gh api repos/barrie-cork/lemmy/pulls/comments/<id>` via `cr_url`) before deciding. |
| cr-2 | major | `.claude/decision-queue.json:3998` | Record explicit user approval for the Task 4 scope decision | Task 4 was **pre-satisfied** via Task-2 scope-bleed; advisor verified spec-conformance + disposed it (runlog 2026-05-17T03:20Z; `v1-AD-e-verify.md` Notes confirm). The user DID approve the scope at plan-approval gate 1 (DQ #237=(a) Dashboard+Audit-only, user-resolved `7d8f84dfc`). Decide: is the approval already recorded (→ `rebut`/`done` citing `7d8f84dfc` + runlog) or genuinely missing an explicit Task-4-scope DQ note (→ `fix-in-pr`: append a DQ `kind: "log"` entry recording the disposition)? This is the most consequential call — read the runlog 2026-05-17T03:20Z entry + verify report Notes before bucketing. |
| cr-3 | nit | `.claude/runlog/v1-AD-e-runlog.md:424` | Clarify the append-only ordering convention | Runlog is append-only chronological. CR wants the convention stated. Low-stakes documentation nit on a meta-file. Likely `wont-fix` (intentional, self-evident from timestamps) or a trivial `fix-in-pr` one-liner — your call per the revert test. |
| cr-4 | **major** | `crates/api/api/src/governance/admin_dashboard_html.rs:77` | Check the feature flag **before** admin authorisation to preserve 404-off semantics | **Substantive code finding — read carefully.** The plan's R-html-3 requires flag-off → **404, not 403**. If the handler checks admin-auth BEFORE the `html_pages_enabled` flag, a non-admin hitting a flag-disabled page gets 403 (leaks "you're not admin") instead of 404 (page doesn't exist). `v1-AD-e-verify.md` Story 3 verified flag-off→404 passes in e2e — so either the ordering is already correct (→ `rebut` citing the passing `admin_html_pages_flag_off_returns_404` test + the actual code order) OR the e2e only tests the admin path and a non-admin+flag-off path is genuinely mis-ordered (→ `fix-in-pr`). **Read the handler source at line 77 via the cr_url diff hunk AND cross-check against the e2e assertion** before deciding. Do not rebut on the test's existence alone if the test doesn't cover non-admin+flag-off. |
| cr-5 | **major** | `crates/api/api/src/governance/admin_dashboard_html.rs:331` | Apply scrub/redaction on user-visible audit strings before rendering | **Substantive code finding.** Audit-tail strings rendered into HTML — if unscrubbed, potential info-leak / XSS surface in admin-visible config-change values. Weigh: does the v1-AD-d JSON layer already scrub (this phase is "pure-templating over shipped v1-AD-d")? If the underlying audit data is already redacted upstream and maud auto-escapes HTML, the render is safe (→ `rebut`/`wont-fix` citing maud's escaping + upstream scrub). If raw audit values reach the template unescaped, it's a real defect (→ `fix-in-pr`). Read the render code at line 331 + check whether maud's `{}` interpolation auto-escapes here. |
| cr-6 | **major** | `crates/server/tests/e2e.rs:15036` | Add a non-admin rejection test for `/api/v4/governance/admin/audit/view` | **Test-coverage finding.** `v1-AD-e-verify.md` Story 2 confirms `admin_audit_html_returns_html_for_admin` (admin 200) exists. CR wants the **non-admin 403** counterpart for `/audit/view` specifically. Check the e2e: does a non-admin-rejection test for `/audit/view` exist (the dashboard path has `admin_dashboard_html_forbidden_for_non_admin` — is there an audit equivalent)? If absent, this is a legitimate `fix-in-pr` (add the symmetric test). If present under a different name, `rebut`/`done` citing it. Read e2e.rs around the audit tests before deciding. |

You are NOT required to reach a specific bucket per row — these are the
considerations, not the answers. Apply the revert test and your reading
of the actual code/diff. The 3 code findings (cr-4/5/6) deserve the most
scrutiny: misbucketing a real major code defect as `rebut`/`wont-fix` is
the exact failure `feedback_coderabbit_block_merge_critical.md` (3×
caught) exists to prevent.

### §2c Outputs

1. **Update `.claude/PRPs/reviews/pr-133-findings.yaml` in place** per
   Phase 3: set each `bucket` to your derived value; write `rationale`
   for every finding (1-2 sentences — what bucket + why; mandatory for
   `rebut`/`wont-fix`, and write one for `fix-in-pr`/`done` too so the
   gate-3 reviewer sees the reasoning); recompute `counters`; **recompute
   the top-level `recommendation`** per Phase 3's note (approve /
   request-changes / block — a PR with open `fix-in-pr` major findings is
   `request-changes` or `block`; all-rebut/wont-fix/done → `approve`).
2. **Draft `.claude/PRPs/reviews/pr-133-comment.md`** per the Phase 4
   template (recommendation + disposition table + per-bucket sections).
3. **Do NOT post the comment.** Do NOT run `gh pr comment`. Do NOT run
   `gh issue create`. The advisor surfaces the four-bucket counts to the
   user at **user gate 3** first; posting/issue-filing happens only after
   that gate, driven separately.

## §3 Required reading

- `.claude/commands/bm/bm-triage.md` — bm-triage operational script.
  Follow Phases 1→4 only (Phase 1 load YAML, Phase 2 four-bucket
  classification, Phase 3 YAML update + recommendation recompute, Phase 4
  draft comment). **STOP after Phase 4** — Phases 5-7 (ASK/post/issue)
  are advisor-gated at gate 3, not yours to execute. Still do Phase 8
  (runlog append) + Phase 9 (output).
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (stable `id`,
  `bucket` enum `fix-in-pr|rebut|carry-forward|done|wont-fix`, `rationale`,
  `addressed_in`, regenerated `counters`, top-level `recommendation`).
- `.claude/lessons/feedback_pr_review_triage_pattern.md` — four-bucket
  heuristics.
- `.claude/lessons/feedback_severity_labels_dont_imply_semantic.md` —
  the revert test ("if I revert this, does the symptom return?").
- `.claude/lessons/feedback_coderabbit_block_merge_critical.md` — why
  misbucketing a real major down to rebut/wont-fix is the cardinal sin.
- `.claude/rules/decision-queue.md` — the "Do not rewrite historical
  entries / forward-only" rule (load-bearing for cr-1/cr-2 bucketing).
- `.claude/PRPs/reports/v1-AD-e-verify.md` — Story 2/3 + Notes
  (Task-4 pre-satisfied disposition; flag-off→404 e2e pass) — context
  for cr-2/cr-4/cr-6 intentional-vs-defect classification.
- `.claude/runlog/v1-AD-e-runlog.md` — full task history incl. the
  2026-05-17T03:20Z Task-4 scope-bleed disposition (cr-2 context).
- `.claude/rules/branch-manager.md` — BM autonomy bounds + file-ownership.
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on
  every gh command.
- Each finding's `cr_url` in the YAML — read the full CR review text via
  `gh api repos/barrie-cork/lemmy/pulls/comments/<comment-id> --jq .body`
  (extract the numeric id from the `#discussion_r<ID>` suffix) for the
  3 code findings (cr-4/5/6) **before** bucketing them. For cr-4/cr-5
  also read the cited source file region; for cr-6 read the e2e.rs
  audit-test region. **Read-only** — do NOT edit `crates/**` or
  `tests/**` (HARD boundary, see §4).

## §4 Constraints

- **`--repo barrie-cork/lemmy`** on every `gh` command.
- **Re-derive ALL 6 buckets from scratch** (§2a) — the pre-set
  `fix-in-pr` values are a poll-cr miss; do not trust them.
- Every finding gets a `bucket` ∈ {`fix-in-pr`, `rebut`,
  `carry-forward`, `done`, `wont-fix`} + a written `rationale`.
- `bucket: fix-in-pr` → `addressed_in: null` (impl fills on fix commit).
- `bucket: rebut` → `rationale` MUST cite the specific rule / decision /
  test / commit SHA that makes CR wrong (e.g. "per
  `.claude/rules/decision-queue.md` forward-only rule" or "covered by
  `<test_fn>` at e2e.rs:<line>" or "user-approved at `7d8f84dfc`").
- `bucket: done` → `rationale` cites the existing commit SHA / test that
  already addresses it; set `addressed_in: <short-sha>`.
- `bucket: wont-fix` → `rationale` cites the reason (intentional / style
  / fork-divergence / append-only-immutable).
- `bucket: carry-forward` → leave `notes: null` for now; do **NOT**
  `gh issue create` (that's a gate-3-confirmed outbound action). Note in
  the runlog which findings you'd carry-forward so the advisor can ASK
  the user at gate 3.
- Recompute `counters` from `findings[]` (the open/done/rebutted/
  carry_forward/wont_fix sub-counts per severity + `by_source` + `total`).
- Recompute top-level `recommendation` (Phase 3): `block` if any
  `fix-in-pr` critical; `request-changes` if any `fix-in-pr` major;
  `approve` if no open `fix-in-pr` critical/major. (0 critical here, so
  it hinges on whether any major lands in `fix-in-pr`.)
- Write the YAML with `encoding="utf-8"`, `allow_unicode=True`,
  `sort_keys=False` (preserve key order; CR severity may carry emoji in
  summaries — `feedback_python_utf8_encoding_windows.md`).
- Draft `.claude/PRPs/reviews/pr-133-comment.md` per Phase 4 template
  (plain Markdown, recommendation + disposition table + per-bucket
  sections). Do NOT post it.
- **Commit the triage YAML + comment draft atomically** (one commit,
  both files) then push to `phase-v1-AD-e`:
  `git add -f .claude/PRPs/reviews/pr-133-findings.yaml .claude/PRPs/reviews/pr-133-comment.md`
  → `git commit -m "chore(bm): triage #133 — four-bucket classification + digest draft"`
  → `git push origin phase-v1-AD-e`. (Both YAML + comment are gitignored
  by default — force-add both. The advisor reads them off the phase
  branch for gate 3.)
- Append the Phase 8 runlog entry to `.claude/runlog/v1-AD-e-runlog.md`
  (`## bm: triage — <ISO>` with bucket counts + recommendation +
  "comment NOT posted (advisor gate 3)" + any carry-forward candidates).
- **STOP after Phase 4** — do NOT execute Phase 5 (ASK), Phase 6
  (gh issue create), or Phase 7 (gh pr comment). Those are advisor /
  user-gate-3 actions. Still do Phase 8 (runlog) + Phase 9 (output the
  bucket-count table back).
- Do **NOT** post any PR comment or send a Telegram ping (advisor
  handles outbound after gate 3).
- Do **NOT** touch `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**` (BM file-ownership HARD
  boundary). Reading code via `gh api` / `git show` for triage judgment
  is fine; editing it is not.
- **Refusal cases (per command):** if a finding looks like an
  ADR-violation being pushed to `rebut`/`wont-fix` without a superseding
  citation → STOP, file a `kind: "blocker"` DQ (do NOT improvise the
  bucket). None of these 6 is flagged as an ADR-violation, so this
  should not fire — but the gate stands.
