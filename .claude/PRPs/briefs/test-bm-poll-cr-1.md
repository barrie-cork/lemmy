# bm-poll-cr — PR #195 initial poll (test dogfood sandbox)

[role:bm-task] test bm-poll-cr-1 — initial poll PR #195 CodeRabbit + Copilot — see .claude/PRPs/briefs/test-bm-poll-cr-1.md

## 1. Role

`[role:bm-task]` — branch-manager verb `bm-poll-cr` per `.claude/commands/bm/bm-poll-cr.md`. First poll of CodeRabbit on PR #195 (`phase-test → governance-v0`).

## 2. Scope

Poll PR #195 for CodeRabbit and Copilot review comments. Create `.claude/PRPs/reviews/pr-195-findings.yaml` from scratch (this is the first poll — no prior YAML exists). Bucket all findings into the four-bucket schema.

**PR #195 phase-branch:** `phase-test`, tip `50195e511` (merge-commit `70b2cc506` with impl `3553e1ba5`).

**Change summary:** adds `crates/utils/src/sandbox.rs` (pure `sandbox_clamp(value: u32, max: u32) -> u32` helper + unit tests) and one `pub mod sandbox;` line in `crates/utils/src/lib.rs`. Zero governance logic, zero migrations, zero e2e edits. Only expected findings: style nits or "this module is unused outside tests" comments (all wont-fix for this dogfood sandbox).

**PR was opened:** by bm-pr Junior #656. Wait ≥ 5 min after PR creation before fetching CR comments (CR may not have posted immediately — use `--comments` flag to check). If CR has not posted after polling `gh pr view --repo barrie-cork/lemmy 195 --json reviews,comments`, wait and note `last_poll_at` with a pending status.

## 3. Required reading

- `.claude/commands/bm/bm-poll-cr.md` — the bm-poll-cr verb script
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (`bucket`, `addressed_in`, `last_poll_at`, `poll_count`)
- `.claude/rules/branch-manager.md` — file-ownership boundaries
- `.claude/lessons/feedback_pr_review_triage_pattern.md` — four-bucket discipline
- `.claude/lessons/feedback_verify_automated_reviewer_claims_against_compiler.md` — CR/Copilot claims are hypotheses

## 4. Constraints

1. **NEVER post a PR comment** without explicit confirmation. Draft any comment into `.claude/PRPs/reviews/pr-195-comment.md` but do NOT `gh pr comment`.
2. **NEVER submit a PR review** (`gh pr review --approve|--request-changes`). Draft only.
3. **NEVER merge** the PR.
4. **NEVER force-push** anything.
5. **NEVER edit `crates/`, `migrations/`, `tests/`, `docs/brehon-law-inspired-network/`, `.claude/PRPs/plans/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`.**
6. **NEVER write `answered_by: "advisor"` or `approved_by`** in any DQ entry.
7. **Create `.claude/PRPs/reviews/pr-195-findings.yaml`** using the schema from `.claude/PRPs/reviews/SCHEMA.md`. This is a fresh file — do NOT copy from a prior PR's YAML.
8. **`--repo barrie-cork/lemmy` MANDATORY** on every `gh pr` command.
9. **If CR has not posted yet:** set `poll_count: 1`, `last_poll_at: <now>`, leave `findings: []` with a `notes: "CR not yet posted — re-poll needed"` field. Raise a `kind: "log"` DQ entry noting the re-poll need. Do NOT raise a blocker.
10. **For any findings discovered:** assign conservative buckets. This is a dogfood sandbox with a trivial change — `wont-fix` is appropriate for style/dead-code nits on `sandbox.rs`. Only `fix-in-pr` for genuine compilation or correctness issues.
11. **Counters regenerate on every YAML write.**
12. **Write a `chore(reviews):` commit** containing only the findings YAML. NEVER `chore(advisor):`.
13. **HANDOVER trailer mandatory** on the final commit.
14. **Push to the worker branch**; daemon finalize-merges into `phase-test`.

## 5. Success signals

- `.claude/PRPs/reviews/pr-195-findings.yaml` created with correct schema.
- `last_poll_at` set to current ISO 8601 UTC, `poll_count: 1`.
- All findings bucketed (or `findings: []` with pending note if CR not yet posted).
- Final commit subject: `chore(reviews): bm-poll-cr-1 — PR #195 — initial CR poll (test dogfood)`.
- HANDOVER trailer present.
- Push to worker branch succeeds.

## 6. Out of scope

- Posting any PR comment or review (drafts only).
- Fix-impl tasks — this is a poll-only step.
- Bm-merge (separate step, user gate 5).
