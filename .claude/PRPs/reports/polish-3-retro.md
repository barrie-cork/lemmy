---
phase: v0-polish-3
date: 2026-04-19
branch: polish/docs-sweep
base: governance-v0 @ 7a0efdf06
shipping_session: primary worktree (advisor); headless agent halted at commit step
---

# polish-3 retro — docs sweep (GH #38 #39 #50 #51 #52)

## What worked

- **Brief was self-contained.** All five GH issues had clear scope, file paths, and (for #38) line-number anchors. The agent worked through them serially without needing the decision queue.
- **GH issue body was authoritative when narrower than the brief.** GH #51 said "Update DQ-6.7 `answer` field"; the brief said "rewrite question and answer." The agent followed the issue (narrower, more specific) and only rewrote the answer. Question text was already accurate. Saved a 200-word edit that would have introduced churn for no signal change.
- **Sweep-vs-spec on GH #50.** Brief named two untagged fences (lines 97, 848) and said "any other untagged fences." A grep sweep found two more (199, 914). All four were tagged. This is the right pattern: brief lists the known cases, agent fills in the implicit "and similar."
- **JSON encoding rule was loud enough to land correctly.** `feedback_python_utf8_encoding_windows.md` plus the brief's "use `Edit` tool directly on JSON string fields" both pointed at the same outcome. The agent edited the JSON via `Edit`/`sed`, never via Python — `§` and `—` round-tripped without cp1252 corruption.

## What surprised us

- **Headless agents inherit allow-list state from the worktree they boot in, not from the user's session.** The polish-3 worktree had no `.claude/settings.local.json` (the file is gitignored), so the agent had no permissions allow-list. Default behaviour without a list is to prompt; in headless mode there is no human to prompt, so `Edit`/`Write` against `.claude/` paths auto-denied silently. The agent diagnosed the symptom correctly ("denied wholesale") but mis-attributed the cause to a deny rule. Real cause: absence of an allow rule that the primary worktree has accumulated over months.
- **Validation passed cleanly with zero new rustdoc warnings.** `cargo doc --workspace --no-deps` produced 19 pre-existing upstream rustdoc warnings (e.g. `[[Instance]]` ambiguous link, bare URLs in `db_schema/lib.rs`) and zero new ones from polish-3's docstring edits. The fork-local citation rewrites in `accept_jury_assignment.rs` and `governance_case/src/impls.rs` are valid markdown.
- **Decision-queue id 38 had a long answer with two diffs in flight.** Editing one paragraph inside a 600-word JSON string field worked fine via `Edit`'s exact-match anchor. The cp1252 trap never materialised because we never went through Python.

## Carry-forward

- **Add `scripts/brehon/cargo-doc.bat` wrapper.** Currently no docs-validation wrapper. The polish-3 agent worked around this by cloning `cargo-check.bat` to a temp file, validating, then deleting. A permanent wrapper would simplify any future docs-only PR. Belongs in a polish-N or pre-tag PR. Not blocking anything.
- **Document the `.claude/settings.local.json` worktree-bootstrap requirement.** When advisors cut a fresh worktree for a headless agent, they should copy `settings.local.json` alongside `.env` (already documented). Otherwise the agent will hit silent Edit/Write denials and either work around them via Bash heredoc (works for some paths) or halt cleanly (which is what polish-3 did — correct behaviour, but cost an iteration). Belongs in `.claude/rules/multi-session-worktree-safety.md` as a sibling of the .env note. Filed as carry-forward, not in this PR.
- **Issue body trumps brief paraphrase when more specific.** Reinforce in `prp-issue-fix` skill or as a feedback memory: when the brief says "rewrite X and Y" but the GH issue body specifies only "update Y", trust the issue. Briefs are summaries; issues are spec.
- **`cargo doc` baseline cleanup is its own polish ticket worth filing.** 19 pre-existing rustdoc warnings is enough to be noisy without being load-bearing. A separate `polish-N-rustdoc-baseline` ticket would clear them in one pass and let future `cargo doc` runs use `-D warnings` as the gate. Not for this PR.