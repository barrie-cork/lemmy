---
name: Watchpoint specificity gate — every plan §4 watchpoint must name a concrete table/file/line
description: At plan-approval, reject any watchpoint that names only a concept ("watch for trait drift") instead of a specific table, file, or schema.rs line — file a DQ requesting revision before approving
type: feedback
originSessionId: reconstructed-2026-06-04-v1-closeout-phase1d
---

> **Reconstruction note (2026-06-04, v1-closeout Phase 1d):** this lesson was cited by the always-load rule `.claude/rules/advisor-orchestrator.md` §3.5 (the watchpoint-specificity gate) and indexed in MEMORY.md, but the file was absent on disk (the GAP-2 broken-citation class). Reconstructed from the §3.5 rule text + the watchpoint-revision DQ history. Behaviour is unchanged — §3.5 already operationalises it; this file is the cited backing.

Rule: at the **plan-approval gate** (advisor `/auto-phase` stage after planning ships, per `advisor-orchestrator.md` §3.5), walk every watchpoint in the plan's §4. Each one MUST cite a **specific, greppable anchor** — a named table, a file path, a `schema.rs` line, a struct/enum/function name, a migration number. A watchpoint that names only a *concept* ("watch for trait drift", "be careful about the federation boundary", "mind the lock ordering") is NOT actionable: the impl-task subagent can't grep a concept, and the advisor can't verify at merge whether the concern was addressed. Concept-only watchpoint → **file a DQ pending from advisor** (`chore(decision-queue): advisor noted vague watchpoint — <slug>`) requesting the planner revise it to name the concrete site, and **hold plan approval** until the revision lands.

**Why:** a watchpoint is a promise the plan makes to its own reviewer — "here is the specific place this change could go wrong; check it." If the promise is a concept, it's unfalsifiable: at `/brehon-verify` or CR-triage time there's no way to confirm the watched thing was handled, so the watchpoint silently does nothing and the risk it was meant to flag rides through unguarded. The whole point of the §4 watchpoint list is to give the post-impl gates a concrete checklist; a concept-only entry is checklist theatre. Examples of the bad→good transform:
- ❌ "watch for trait-bound drift" → ✅ "watch that `GovernanceCase<S>` still implements `TryFrom<ModerationCase>` in `crates/api/api/src/governance/mod.rs` after the retrofit — `cargo check -p lemmy_api` must pass"
- ❌ "mind the schema change" → ✅ "watch that `person.membership_state` stays in column order matching `person::all_columns` in `crates/db_schema/src/schema.rs:NNN` (Diesel asserts tuple order at compile time)"
- ❌ "be careful with the lock" → ✅ "watch that `pg_advisory_xact_lock` in `sponsor_liability_grace.rs:135` is acquired ABOVE the quorum gate, not inside it"

**How to apply:** this is a mechanical pre-approval pass, not a judgment call. For each §4 watchpoint:

1. Does the text contain a concrete anchor — a path, a `table.column`, a `file:line`, a type/fn name, or a migration id? `grep`-test it: could you turn this watchpoint into a one-line `grep`/`cargo check` command? If yes → pass.
2. If it names only a concept with no anchor → file the revision DQ, cite this lesson + §3.5, hold approval.
3. Surface the count in the plan-approval message: "watchpoints: N specific ✓, M concept-only → DQ filed for revision."

The planner reconstructs the anchor (it knows which code the watchpoint was about); the advisor's job is only to enforce that the anchor is *present* before approving. A plan that ships to impl with concept-only watchpoints is a process miss the retro flags.

## See also

- `.claude/rules/advisor-orchestrator.md` §3.5 (the gate that enforces this) + §3.4 (the DoD smoke test that runs alongside it at plan approval)
- `feedback_falsifiable_hypothesis_before_structural_fix.md` — the sibling discipline: a watchpoint, like a structural-fix DQ's RCA, is a hypothesis that must be made concrete enough to test
- `.claude/PRPs/templates/plan.template.md` §4 — the watchpoint section this gate audits
