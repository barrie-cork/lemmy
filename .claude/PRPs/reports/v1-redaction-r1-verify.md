# Verify report — v1-redaction-r1

**Run at:** 2026-06-01T~14:00Z (updated 2026-06-01 post-fix-impl-1)  
**Phase branch:** `phase-v1-redaction-r1` @ `df5bf3558`  
**Plan:** `.claude/PRPs/plans/v1-redaction-r1.plan.md`  
**Outcome summary:** 4 stories: 4✓ 0✗-regression 0✗-phantom 0[malformed]

---

## Pre-flight checks

- Tasks 1/2/3 commits confirmed on `origin/phase-v1-redaction-r1`:
  - Task 1: `e42cb4d70 feat(redaction): adversarial test corpus locks docstring invariants`
  - Task 2: `471ba7565 feat(redaction): bounded scrub_json recursion + depth-cap test`
  - Task 3: `71709af59 feat(redaction): regex commentary + Maintenance invariants`
- No Junior tasks running on phase.

---

## Story 1 — Adversarial test corpus locks the docstring-asserted invariants

- **Composing tasks:** Task 1
- **Structural outputs:**
  - ✓ `fn scrub_order_dependence_mention_with_remote_host_not_eaten_by_email` — present
  - ✓ `fn scrub_handles_newline_separated_mentions` — present
  - ✓ `fn scrub_does_not_treat_username_as_regex_pattern` — present
  - ✓ `fn scrub_json_preserves_integer_id_fields` — present
  - ✓ `fn scrub_mention_after_cyrillic_letter_is_scrubbed` — present
  - ✓ 2 `#[ignore = "v1-redaction-r2 ...]` attributes at lines 276, 289 (3rd hit at line 52 is a `//!` doc-comment, not an attribute)
  - ✓ DQ `kind: "log"` entry `642f320508ca-001` from `from: "impl"`, `answered_by: "impl-self-resolved"`, naming v1-redaction-r2 as Unicode carry-forward owner
- **Checkpoint:** `cargo-test.bat -p lemmy_db_schema --features full redaction::tests`
  - Exit: **0** ✓
  - Result: `test result: ok. 11 passed; 0 failed; 2 ignored` — matches expected exactly
- **Outcome:** ✓

---

## Story 2 — Bounded scrub_json recursion prevents stack overflow on adversarial nesting

- **Composing tasks:** Task 2
- **Structural outputs:**
  - ✓ `const MAX_RECURSION_DEPTH: usize = 64` — present at module scope
  - ✓ `fn scrub_json_inner` — present (file-private, no `pub`)
  - ✓ `pub fn scrub_json` — present (1 hit, signature unchanged)
  - ✓ `fn scrub_json_at_depth_cap_returns_null_not_truncated_tree` — present in test module
  - ✓ Caller diff (governance_log.rs, admin_dashboard_html.rs, submit_jury_vote.rs, admin_rule_sets.rs) = 0 lines — no callers modified
- **Checkpoint:** `cargo-test.bat -p lemmy_db_schema --features full redaction::tests::scrub_json_at_depth_cap_returns_null_not_truncated_tree`
  - Exit: **0** ✓
  - Result: `test result: ok. 1 passed; 0 failed` — matches expected exactly
- **Outcome:** ✓

---

## Story 3 — Policy commentary documents the over-scrub bias for future maintainers

- **Composing tasks:** Task 3
- **Structural outputs:**
  - ✓ `//! ## Maintenance invariants` heading present
  - ✓ `ADR-015` citation present (multiple hits in doc-comments)
  - ✓ `GDPR §17` citation present
  - ✓ `MAX_RECURSION_DEPTH` citation present
  - ✓ `v1-redaction-r2` citation present
  - ✓ Per-regex `//` comments present above `mention_regex` and `email_regex`
  - ✓ Targeted clippy `-p lemmy_db_schema --features full --no-deps -- -D warnings`: exit **0** — redaction.rs changes are clean
- **Checkpoint (plan §16a):** `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings`
  - Exit: **0** ✓ (post-fix-impl-1 @ `df5bf3558`)
  - Fix-impl-1 commit `df5bf3558` (cherry-picked from Junior task #564 worker branch) resolves 6 pre-existing lints in `reputation_snapshot.rs`: `map_or(false, ...)` → `is_some_and(...)` + 5 `as` casts → `try_from().unwrap_or(MAX)`
  - Log: `.claude/PRPs/debug/v1-redaction-r1-clippy-fix-impl-1.log`
- **Outcome:** ✓

---

## Story 4 — Call-site audit confirms zero bypass paths into the three GDPR-gated columns

- **Composing tasks:** done at plan-author time; re-verified by Task 0 Probe 6
- **Structural outputs:**
  - ⚠ `.claude/PRPs/debug/v1-redaction-r1-callsite-audit.log` — ABSENT on phase branch (`.gitignore` excludes `*.log`; plan-author expected this to be committed, but Task 0 has no commit per plan design). Verified locally instead.
  - ✓ Callsite count: 32 hits (grep `scrub(|scrub_json(` across `crates/ --type rust`) — exceeds threshold of ≥21
  - ✓ 3 write-path chokepoints confirmed: `governance_log.rs:268`, `submit_jury_vote.rs:527+531`, `governance_log.rs` (all scrub before append)
  - ✓ New RT-r5 callers (`admin_sponsor_allowlist.rs:100,186`) are write-path with `scrub_json` — correct use, not bypass
  - ✓ No bypass paths found (all callers scrub before writing to gated columns)
- **Checkpoint:** `rg "scrub\(|scrub_json\(" crates/ --type rust | wc -l`
  - Result: **32** lines — above threshold of 21 ✓
- **Note on Brief-Scope phantom:** The plan names the audit log as a Brief-Scope output "committed by Task 0", but Task 0's FILES YAML correctly has `creates: []` and makes no commit. The log is gitignored. This is a plan-inconsistency (plan prose vs FILES YAML), not a phantom — the underlying audit was performed (locally, during Task 0 execution) and the count passes. Filing a DQ log entry to note the plan prose drift.
- **Outcome:** ✓ (with plan-inconsistency note)

---

## Required actions

### Story 3 — RESOLVED

fix-impl-1 (#564) fixed 6 pre-existing lints in `reputation_snapshot.rs`. Workspace clippy now clean. No further action.

### Story 4 plan-inconsistency

Plan §16a Story 4 says the callsite audit log was "committed by Task 0". Task 0's FILES YAML says `creates: []`. The log is gitignored (`*.log`). No phantom (the audit was done), but the plan prose is inconsistent with the FILES YAML. Will file a `kind: "log"` DQ noting this for retro harvest.

---

## E2e validation

Separate from §16a stories: full e2e suite ran against `9bf49615f` (post-merge-forward + e2e.rs DISABLE-test fix):
- **128 passed; 0 failed; 5 ignored** — E2E_EXIT_0 ✓
- Log: `.claude/PRPs/debug/v1-redaction-r1-e2e-final.log`
