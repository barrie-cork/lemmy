# Brief: m1-b BM-POLL-CR

## 1. Role + dispatch

`[role:bm-task] m1-b-bm-poll-cr — see .claude/PRPs/briefs/m1-b-bm-poll-cr-1.md`

## 2. Scope

Poll CodeRabbit findings on PR #177 (`phase-m1-b → governance-v0`) on `barrie-cork/lemmy`.

**Produce:**
- Findings YAML at `.claude/PRPs/reviews/pr-177-findings.yaml` (create new — first poll on this PR)
- Runlog entry in `.claude/runlog/m1-b-runlog.md`
- Summary of finding counts by bucket + severity in task output

**Do NOT:**
- Merge the PR
- Post any PR comments or submit reviews
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `docs/`, `tests/`, `migrations/`

## 3. Context

CodeRabbit has completed its review: **"Actionable comments posted: 3"**. The review is fully posted BEFORE this bm-poll-cr runs — no wait required; proceed directly to YAML authorship. Ingest all 3 actionable comments (plus any nitpick/outside-diff findings CR folded into collapsible sections) into the findings YAML.

PR scope reminder (M1-b Tree B, Tasks 1–7): the new `governance_messaging_config` migration + Diesel model + admin DTOs + admin handler + identity-policy validator + `bridge_notify` fire-and-forget POST + 2 e2e tests. CR findings should focus on Rust quality (error handling, type shape, the fire-and-forget swallow path, the validator's scope-prefix matching), migration correctness, and e2e assertion shape.

## 4. Required reading

- `.claude/rules/branch-manager.md` — file-ownership boundaries, autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema

## 5. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- PR number: 177
- Record poll results in `.claude/runlog/m1-b-runlog.md`
- Do NOT post comments or submit reviews (those are separate confirm-gated verbs)
- Initial bucket: default `fix-in-pr` for CRITICAL/MAJOR; flag `rebut`/`carry-forward`/`wont-fix` candidates for advisor triage at gate-3 (do not finalize buckets — bm-triage + advisor own that)
- `source: coderabbit` on every finding; stable `id` per finding; `severity` ∈ {critical, major, medium, low, nit}
