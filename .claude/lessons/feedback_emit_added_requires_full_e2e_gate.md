---
name: Emit-path additions require a phase-tip full-binary e2e gate
description: A plan that adds a new reputation/governance_log emit path under crates/api/api/src/governance/**.rs (or the handlers they call) MUST mandate a whole-binary `cargo test --test e2e` phase-tip gate in §15 — not just per-test single-fn gates. RT-r3 shipped a vote-outcome emit with only per-test gates; 4 downstream count-asserting e2e tests went stale to trunk and required the entire v1-rt-r3-followup mini-phase to fix.
type: feedback
---

# Emit-path additions require a phase-tip full-binary e2e gate

When a plan adds a **new emit path** — a new `reputation_event` row,
a new `governance_log` `entry_kind`, or any side-effect that an e2e
test counts or asserts on — the §15 DoD MUST include a **whole-binary**
`cargo test --workspace --test e2e --features full` (NO test-name
filter) run on the phase tip before bm-pr. Per-test single-fn gates
(`--test e2e -- <one_fn>`) are insufficient: they only execute the
tests the plan author *thought* to name, and an emit path radiates to
**every** test that happens to count that table — including tests in
sibling modules the plan never mentions.

**Why this lesson exists:** RT-r3 (`996765cae feat(governance):
vote-outcome + evidence-cited emit in submit_jury_vote (task 2)`) added
two new emit paths:

- **Source 3** (vote-outcome): `+ParticipationConsistency` reputation_event
  per majority-aligned juror + a `vote_outcome_recorded` governance_log
  kind, on `submit_jury_vote`.
- **Source 4a** (evidence-cited): `+ReportingAccuracy` + an
  `evidence_quality_recorded` kind, gated on seeded `case_evidence`.

RT-r3's §15 ran per-test gates only. The new `+3 ParticipationConsistency`
rows landed in the **filterless** `reputation_event::table.count()`
assertions of **four** tests across three different modules
(`report_to_modlog_golden_path`, `governance_log_sequence_matches_prd_state_machine`,
and two `mod v1_sl_d_fixtures::submit_jury_vote_*` fixtures), none of
which RT-r3 touched. RT-r3 shipped green (its named tests passed) but the
phase-tip e2e suite was actually `115 passed; 4 failed; 5 ignored` — the
4 failures survived to `governance-v0` undetected because no whole-binary
gate ran. Fixing them cost an entire follow-up mini-phase
(`v1-rt-r3-followup`: plan + 3 tasks + 2 e2e runs).

**The rule (mechanical):** in plan authorship + the advisor's §3.4 DoD
smoke test, if the plan's IMPLEMENT touches an emit site
(`emit_reputation_event`, a new `ENTRY_KIND_*` constant, or a handler
that calls them under `crates/api/api/src/governance/**.rs`), the §15
DoD MUST contain the whole-binary `--test e2e` line. A plan that adds an
emit path but only lists `--test e2e -- <specific_fns>` in §15 is a
process miss the planning-gate (gate 1) should catch.

**Detection at retro:** grep the shipped plan's §15 for `--test e2e`
with no trailing `--` filter. If the diff added an emit path and §15
only has filtered runs, flag it.

**See also:**

- `.claude/lessons/feedback_phase_2_e2e_gate_enforcement.md` — the
  companion procedural gate (bm-pr refuses without a Phase-2 e2e
  `validate-pending` pass). That lesson catches "e2e never ran at all";
  THIS lesson catches "e2e ran but only on a hand-picked subset, missing
  the cross-module blast radius of an emit change".
- `.claude/PRPs/reports/v1-rt-r3-followup-retro.md` — the mini-phase this
  miss produced.
