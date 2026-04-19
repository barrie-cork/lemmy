# polish-3 (docs sweep) — impl brief

**Authored by:** advisor (homeserver session), 2026-04-19 post-polish-1-merge
**For:** whoever picks up polish-3 next
**Base:** `governance-v0` HEAD `155eb8d18` (PR #64 merge)
**Branch to cut:** `polish/docs-sweep`

## Scope

Five minor docs fixes, bundled into one PR. Zero code compile surface — all markdown + one JSON string edit + two Rust docstring edits.

### GH #38 — fork-local design-doc citations + broken relative link

Three files:
- `crates/api/api/src/governance/accept_jury_assignment.rs` lines 7, 12 — `//!` docstring — replace `plan §11.X` refs with `Phase 5c task N` form
- `crates/db_views/governance_case/src/impls.rs` lines 171, 187 — `///` docstring — same substitution pattern
- `.claude/PRPs/reports/phase-5c-complete-report.md` line 27 — fix relative link (broken path)

**Note:** Rust doc edits feed `cargo doc` but not `cargo check`. Keep edits plain-text; no `#[doc = ...]` attribute touches.

### GH #39 — `SUBSCRIPTIONS.md` serial-id gap semantics under rollback

One file: `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md`

**Scope:** add ~4–6 line paragraph (or 3 bullets) near the "Catch-up on subscriber start" section clarifying that serial-id gaps under rollback are expected and how subscribers should handle them. Exact wording in #39 body.

### GH #50 — Phase 6 plan MD040 fences + stale enum type names

One file: `.claude/PRPs/plans/phase-6-federation.plan.md`

**Two things:**
- Add language tags to fence blocks at lines 97, 848 (and any other untagged fences in the file — check with markdownlint MD040 if available, else grep for triple-backtick with no language)
- Fix enum type names around lines 480/492/493: `*_enum` → plain names (`attestation_type_enum` → `attestation_type`, `sanction_action_enum` → `sanction_action`, `sanction_scope_enum` → `sanction_scope`)

### GH #51 — DQ-6.7 wording stale

One file: `.claude/decision-queue.json`

**Scope:** entry `id: 38` — update `question` and `answer` text. Current text says "fresh pool conn"; actual Phase 6 resolution used "in-flight conn." Rewrite the string values to match reality.

**Encoding trap:** per polish-1 runlog + `feedback_python_utf8_encoding_windows`, if you edit this JSON via Python, pass `encoding="utf-8"` on open/write. **Recommendation:** use the `Edit` tool directly on the JSON string fields — avoids the Python-encoding trap entirely.

### GH #52 — `task-hopper.md` rule contradiction

One file: `.claude/rules/task-hopper.md` lines 149–153 / 177 / 181 area

**Scope:** rule says "never hand-edit" then describes a crash-recovery hand-edit path. Two options:
- **Option B (preferred, faster):** remove the agent-facing "or edit JSON" line at 177; keep advisor hand-edit carve-out at 181 with clearer "advisor-only" framing
- **Option A (rejected for polish-3):** route recovery through `task-hopper.sh escalate` — pulls the fix out of docs-only scope and into code

Use Option B.

**Auto-load note:** this rule auto-loads in `claude -p` mode. Keep edits prose-only (no hook YAML, no slash-command invocation strings).

## Shipping rules (from PR #64 / PR #46 retros)

- Base `governance-v0` at `155eb8d18`
- Merge method: `--merge` (never `--squash`)
- One commit: `chore(docs): v0-polish docs sweep — GH #38 #39 #50 #51 #52`
  - Alternative (per-issue commits) is fine if CodeRabbit review prefers it; diff stays small either way (~60–90 lines across 7 files)
- One retro at `.claude/PRPs/reports/polish-3-retro.md`
- Critical-only CodeRabbit discipline as with polish-2

## Validation sequence

**Mostly none.** Docs-only changes:

1. `cargo doc --workspace --no-deps` — verify #38 Rust docstring edits don't break `cargo doc` (malformed link in a docstring would fail it)
2. `python3 -c "import json; json.load(open('.claude/decision-queue.json', encoding='utf-8'))"` — verify #51 JSON still parses
3. Eyeball `task-hopper.md` render (no CI gate for this, but visually confirm the rule reads cleanly after Option B edit)

No cargo check / clippy / e2e needed — these don't touch compile surface.

## PR shape

PR title: `v0-polish-3: docs sweep — GH #38 #39 #50 #51 #52`

PR body: list each issue + one-line scope + the file(s) touched. Close all 5 issues via `Closes #38, #39, #50, #51, #52` in PR body.

## Out-of-scope

- GH #6 (stale plan-drift report) — polish-1 runlog (`00-polish-intake.md § Recommendation 2`) flagged this as "verify-and-close in polish-3 or pre-tag." **Not in polish-3.** Keep polish-3 focused on the named 5. Either close #6 separately after a plan-drift re-run on `155eb8d18`, or defer to pre-tag check.
- Polish-2 Bucket C items — separate PR, separate brief (`10-polish-2-brief.md`).

## Parallel-safety

- Polish-3 touches: `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md`, two Rust docstring files, plan file, decision-queue.json, task-hopper.md rule, phase-5c report
- Polish-2 touches: `scripts/brehon/task-hopper.sh`, `publish_sanction_notice.rs`, agent-*.md briefs
- **No file overlap.** Land in either order.
