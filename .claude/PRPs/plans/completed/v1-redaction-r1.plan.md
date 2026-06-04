# Plan: v1-redaction-r1 — harden the GDPR identifier-scrubber (regex, recursion bound, policy notes, adversarial tests)

## 1. Summary

A narrow hardening pass on the v0 redaction layer
(`crates/db_schema/src/source/governance/redaction.rs`) closing four
ship-blocker risks the conformance audit
(`.claude/PRPs/reports/conformance-audit-redaction-rs-2026-05-28.md`)
surfaced as H1–H4: an asserted-but-untested ordering invariant on
`scrub`, an unbounded `scrub_json` recursion, a Unicode boundary-class
edge that no test exercises, and an implicit over-scrub policy on
`email_regex` that a future maintainer could "tighten" into a GDPR
violation. Ships as one PR — the four classes are mechanically
interlinked (a recursion-bound change shifts test invariants, a regex
tightening shifts adversarial expected outputs). Adversarial coverage
lands as **unit tests in `redaction.rs`'s existing `#[cfg(test)] mod
tests`**, NOT in e2e (the existing 5-test happy-path module is the
canonical sibling shape to mirror per
`feedback_plan_stub_uniformity_with_canonical_sibling.md`). Headline
acceptance: every §16a story `[done]` against `/brehon-verify`; the
existing 5 redaction unit tests pass unchanged; new adversarial
coverage locks the docstring-asserted order-dependence + Unicode
confusable handling + nested-JSON recursion cap; new commentary
documents over-scrub bias per GDPR §17 + ADR-015.

## 2. Source

- `.claude/PRPs/briefs/v1-redaction-r1-planning-1.md` @ `1edb8b94c`
- `.claude/PRPs/reports/conformance-audit-redaction-rs-2026-05-28.md`
  @ `1edb8b94c` (H1–H4 four hypotheses; six-axis: 0 findings — file is
  pure-Rust regex + tree-walk outside the framework's defect class)
- GH #58 — P0 GDPR ship-blocker source issue
- ADR-015 (pseudonymised actor IDs in the governance log) @
  `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md:257-271`
- IMPLEMENTATION-PLAN-v0.md §4.2 (redaction service explicit
  simplification — "regex + email + URL only; runtime-loaded
  display-name blocklist is v1") @
  `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md:468-476`
- 06-security-and-threat-model.md §6.1 (redaction service is a single
  code path; scrub identifiers before append) @ `:355-364`
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A
  discipline (NOT invoked for `redaction.rs` unit tests; the existing
  tests at lines 109-158 use plain `#[test] fn` with `assert_eq!`
  because they're pure-function tests, no async, no DB — per
  `feedback_plan_stub_uniformity_with_canonical_sibling.md` the new
  adversarial tests mirror that shape verbatim, NOT `LemmyResult<()>`).
- `.claude/lessons/feedback_clippy_test_style.md` — `#![deny(unwrap,
  expect)]` for any new test; `assert_eq!` + early returns; no
  `.unwrap()`/`.expect()`/`dbg!`.
- `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md`
  — mirror existing test pattern verbatim (plain `#[test] fn` not
  `LemmyResult`).
- `.claude/lessons/feedback_read_canonical_before_writing_spec.md` —
  Read the existing test module + Read the production fns before
  writing any new test or modifying the recursion shape.
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md`
  — N/A for `redaction.rs` directly (no DB writes); analogue
  applies to Task 2's recursion-bound: the change is semantic-
  preserving for depth ≤ 64 + hard-cap at depth > 64. Document the
  semantic change in commit body.
- `.claude/lessons/feedback_features_full_workspace_only.md` +
  `.claude/lessons/feedback_features_full_p_crate_incompatible.md` —
  every §15 cargo command uses `--workspace --features full`. Brief
  also notes `-p lemmy_db_schema --features full` is **valid here
  because `lemmy_db_schema` defines a `full` feature** (verified at
  plan-author time: `rg '^name = "full"' crates/db_schema/Cargo.toml`
  shows the feature is defined). Use `--workspace --features full`
  for §15 uniformly per R8; the `-p lemmy_db_schema` form is allowed
  only for the targeted unit-test invocation in §14.
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — every
  §13 task carries `creates:` + `modifies:` + `requires:` YAML.
- `.claude/lessons/feedback_plan_dod_dry_run_at_write.md` — every §15
  command dry-runnable by advisor against current HEAD.
- `.claude/lessons/feedback_pre_phase_dod_smoke_test.md` — Task 0 runs
  the harness audit pre-impl.
- `.claude/lessons/feedback_laptop_default_for_validate_pending.md` —
  Shape G suspended; cargo runs on laptop via `validate-pending-
  laptop` + `validate-pending-laptop-e2e` paths.
- `.claude/lessons/feedback_validate_pending_laptop_must_use_wrapper.md`
  — every laptop-side cargo invocation goes through the `.bat`/`.sh`
  wrapper (libpq / vcvars / pkg-config setup).
- `.claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md` —
  e2e must be invoked via `cmd //c "scripts\\brehon\\cargo-test.bat
  --workspace --test e2e --features full"`; never bare `cargo test`,
  never `-p lemmy_server --features full`.
- `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` —
  WP-6 call-site audit folded into planner work (§11 §15.5 §16a
  Story 4), re-verified by Task 0 Probe 6.

## 3. Problem statement

The v0 redaction layer (3 regex factories + recursive `scrub_json`) is
**conservatively correct for the happy path** the existing 5 unit tests
exercise, but **four latent risks** identified by the 2026-05-28
conformance audit could each become a GDPR-incident on pilot launch:

- **No test asserts the order-dependence invariant the module
  docstring promises** (H1 — `redaction.rs:66-73`). The existing test
  at line 134 uses `@bob` (no `@host` tail), which doesn't exercise
  the cited footgun (`@bob@remote.example` → `[redacted]`, not
  `@[redacted]`). A future refactor that re-orders `scrub`'s three
  `replace_all` calls passes the existing tests and breaks the
  docstring contract silently.
- **`scrub_json` recursion is unbounded** (H2 —
  `redaction.rs:88-101`). A malicious or buggy upstream constructing
  a 10K-deep JSON tree blows the stack. `serde_json`'s deserialiser
  default `recursion_limit = 128` has already fired by the time
  `scrub_json` sees the `Value`, but defence-in-depth at the scrub
  layer is missing and the inbound federation oversize cap is the
  only structural limit upstream.
- **Unicode confusable handling is untested** (H3 —
  `redaction.rs:31-43`). The `mention_regex` boundary class
  `[^A-Za-z0-9._%+\-]` is ASCII-only. Non-ASCII letters DO match the
  boundary class (they're in the negated set), so a Cyrillic-preceded
  mention IS scrubbed — but no test asserts this, and adversarial
  edges (zero-width joiner, right-to-left override, mid-handle
  Cyrillic lookalike) are undocumented.
- **`email_regex` over-scrub bias is implicit** (H4 —
  `redaction.rs:45-51`). The regex accepts non-RFC-5322 forms
  (`version-1.0@build-2026` matches). A future maintainer reading
  "this regex is too loose" with no comment explaining the GDPR
  bias might tighten it and introduce a right-to-delete violation.

Additionally per WP-6: every write into the three gated columns
(`public_case_log.summary`, `public_case_log.rationale_redacted`,
`governance_log.payload`) must go through `scrub`/`scrub_json`. The
call-site audit at plan-author time confirms zero bypass paths (see
§15.5 Cross-cutting); Task 0 Probe 6 re-verifies at lane-cut time.

## 4. Solution statement

Three implementation surfaces, all editing one file — sequential, no
`[P]` parallelism (file overlap makes cohort dispatch unsafe per
`.claude/rules/advisor-orchestrator.md` §4.1):

### Surface 1 — Adversarial test corpus (Task 1, addresses WP-1, WP-2, WP-5, WP-7, WP-8)

Extends `crates/db_schema/src/source/governance/redaction.rs`'s
`#[cfg(test)] mod tests` (lines 103-159) with **new test fns
mirroring the existing 5-test pattern verbatim** (plain `#[test] fn`,
`assert_eq!`, no `LemmyResult`, no `.unwrap`/`.expect`). Production
code (lines 1-101) is **unchanged** in Task 1. New tests:

- `scrub_order_dependence_mention_with_remote_host_not_eaten_by_email`
  (locks docstring invariant at lines 66-73; the H1 footgun).
- `scrub_handles_newline_separated_mentions` (WP-7 — `@alice\n@bob\n@carol`).
- `scrub_does_not_treat_username_as_regex_pattern` (WP-8 — `@.*`,
  `@[abc]` literal handling).
- `scrub_json_preserves_integer_id_fields` (WP-5 — confirms
  integer `community_id` survives scrub_json walk).
- `scrub_mention_after_cyrillic_letter_is_scrubbed` (WP-2 — locks the
  ASCII-boundary-class-as-Unicode-permissive behaviour).
- `scrub_zero_width_joiner_between_at_and_handle` **`#[ignore]`** with
  `// TODO(v1-redaction-r2): handle Unicode confusables` comment (WP-2
  — known-deferred adversarial case; the test documents intent without
  blocking ship; v1 carry-forward recorded via `kind: "log"` DQ).
- `scrub_handle_with_unicode_confusable_in_username` **`#[ignore]`** —
  same v1 carry-forward.

### Surface 2 — Bounded `scrub_json` recursion (Task 2, addresses WP-3)

Rewrites `scrub_json` (lines 88-101) to introduce a depth-cap. New
shape:

```rust
const MAX_RECURSION_DEPTH: usize = 64;

pub fn scrub_json(value: &Value) -> Value {
  scrub_json_inner(value, 0)
}

fn scrub_json_inner(value: &Value, depth: usize) -> Value {
  if depth >= MAX_RECURSION_DEPTH {
    // Hard cap; over-scrub bias under GDPR (per ADR-015 +
    // IMPLEMENTATION-PLAN-v0.md §4.2 — "leak of a username
    // defeats right-to-delete permanently"). Truncated tree
    // looking complete is worse than visibly-null leaves.
    return Value::Null;
  }
  match value {
    Value::String(s) => Value::String(scrub(s)),
    Value::Array(items) => Value::Array(
      items.iter().map(|v| scrub_json_inner(v, depth + 1)).collect()
    ),
    Value::Object(map) => Value::Object(
      map.iter().map(|(k, v)| (k.clone(), scrub_json_inner(v, depth + 1))).collect()
    ),
    other => other.clone(),
  }
}
```

Plus one new adversarial test:
- `scrub_json_at_depth_cap_returns_null_not_truncated_tree` —
  constructs a 70-deep nested array of strings; walks down to depth 64;
  asserts the leaf is `Value::Null` (NOT the un-scrubbed inner
  payload). Pattern-tests against reality, not syntax.

The public signature `pub fn scrub_json(value: &Value) -> Value` is
**unchanged** — no caller updates needed. `scrub_json_inner` is a
file-private helper; `MAX_RECURSION_DEPTH` is a module-level `const`.
Per WP-3 the cap value 64 is well below stack-overflow threshold
(~10K on typical x86_64) and generous for legitimate use (current
governance_log payloads are flat — 5-6 levels worst case).

### Surface 3 — Regex policy commentary (Task 3, addresses WP-4)

Pure comment additions above each regex factory (no behaviour change):

- Above `email_regex` (line ~50): policy note citing GDPR §17 +
  ADR-015 — over-scrub bias is intentional; false positives on
  non-email strings matching the pattern are acceptable; false
  negatives are GDPR violations. v1 may consider tighter RFC-5322
  forms after pilot data shows actual false-positive cost.
- Above `mention_regex` (line ~31): boundary-class documentation —
  ASCII-only class is correct for v0 because non-ASCII letters
  fall INTO the negated set (boundary). v1 may add
  `unicode-normalization` (deferred to v1-redaction-r2; tracked via
  `#[ignore]` tests added in Task 1 + `kind: "log"` DQ).
- Module-level doc-comment (lines 1-25): add a "Maintenance
  invariants" subsection enumerating: (a) order-dependence in
  `scrub` is load-bearing (test locks it); (b) `scrub_json`
  depth-cap is defence-in-depth; (c) over-scrub bias on
  `email_regex` is intentional.

No production regex tightening. No new dependencies. No `Cargo.toml`
edit.

## 5. Metadata

- **Phase:** `v1-redaction-r1`
- **Branch:** `phase-v1-redaction-r1` (cut by `bm-cut` before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 5 (Task 0 pre-flight + Tasks 1-3 impl + Task 4 retro)
- **Estimated cargo budget:** `~6 GB peak` per task (single
  `cargo check --workspace --features full`; no migrations; no e2e
  edits in any impl task — phase-tip e2e gate runs once after Task 3)
- **Forbidden-window applicability:** standard (per
  `.claude/rules/advisor-orchestrator.md` §5.1). Shape G **SUSPENDED**
  per `feedback_laptop_default_for_validate_pending.md` →
  `validate-pending-laptop` + `validate-pending-laptop-e2e` paths
  active; cargo runs on laptop. Forbidden-window check applies.
- **Complexity score:** `4/10`

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Sonnet target ⇒
split-DQ fires at `score > 8`; r1 scores `4/10` — no split required.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 3 impl tasks (Tasks 1-3); Task 0 + retro excluded |
| Migrations touched | +2 each | 0 | r1 is unit-test corpus + recursion bound + commentary only |
| Crates touched | +1 each | 1 | `crates/db_schema` only (single file edits across all 3 impl tasks); the `crates/api/api/.../redaction.rs` re-export shim is **NOT TOUCHED** per brief §2 |
| `crates/server/tests/e2e.rs` edits | +3 each | 0 | r1 adds unit tests in `redaction.rs`; no e2e edits |
| New ADR-affecting decisions | +2 each | 0 | r1 hardens existing ADR-015 implementation; no new ADR |
| Cargo budget peak above 6 GB | +1 per GB | 0 | ~6 GB peak; Shape-G suspended (laptop path) |
| **Total** | — | **1** | Calibrated up to **4/10** for narrative + reviewer-judgment factors: GDPR-critical scope; one file with both production + test edits across 3 sequential tasks (commit-discipline risk); known-deferred `#[ignore]` tests + DQ carry-forward. Threshold for split-DQ: `>8` (Sonnet target); no split |

### 5.2 Per-task complexity ceiling (Sonnet target)

Sonnet ceiling: `≤ 4` files per task / `≤ 2` distinct crates per
task; `crates/server/tests/e2e.rs` does NOT appear in `modifies:`
alongside non-test logic.

- **Task 1** (adversarial tests): 1 file
  (`redaction.rs` test module only) in 1 crate. ✓ Under ceiling.
- **Task 2** (recursion bound): 1 file
  (`redaction.rs` production + 1 new test) in 1 crate. ✓ Under ceiling.
- **Task 3** (commentary): 1 file
  (`redaction.rs` comment-only edits) in 1 crate. ✓ Under ceiling.

## 6. Relationship to other v1-redaction-* sub-phases

- **Depends on:** Phase 6 DQ-6.6-inbound (resolved id 37) — moved
  the redaction module from `crates/api/api/src/governance/redaction.rs`
  (now a 22-line `pub use` shim) to
  `crates/db_schema/src/source/governance/redaction.rs` (canonical
  159-line definition). r1 edits ONLY the canonical definition; the
  shim is invariant per brief §2.
- **Coexists with (lane B):** v1-jm-b-followups-r1 per
  2026-05-28 handover. Lane B is separate worktree; daemon
  `.git/index.lock` cohort math: 2 concurrent planning Juniors (Lane
  A + Lane B) below cohort-size-3 threshold per
  `feedback_cohort_shared_git_index_contention.md` (advisor verified
  at brief §10 pre-queue checklist).
- **Followed by:** v1-redaction-r2 (not yet on roadmap) — picks up
  the `#[ignore]`d Unicode confusable tests, adds
  `unicode-normalization` crate dependency for NFKC normalisation
  pre-scrub. Carry-forward recorded via `kind: "log"` DQ at Task 1
  completion.
- v1 planning-queue id: `redaction-r1` (GDPR-critical P0 ship-blocker
  per GH #58).

## 7. Preflight guardrails inherited from prior phases

- **R1 (clippy — `unwrap`/`expect` in tests):** new test fns use
  `assert_eq!` / `assert!` only; no `.unwrap()`/`.expect()`/`dbg!`
  per `feedback_clippy_test_style.md`. The existing test module
  already imports `pretty_assertions::assert_eq` — new tests use the
  same import (already at `redaction.rs:106`).
- **R2 (canonical-sibling shape):** new unit tests mirror the
  existing 5 tests at `redaction.rs:109-158` verbatim — plain
  `#[test] fn`, no `LemmyResult`, no `async`. Per
  `feedback_plan_stub_uniformity_with_canonical_sibling.md`.
- **R3 (silent-failure refusal):** the recursion depth-cap MUST
  substitute `Value::Null` at the cap, NOT silently return the
  truncated tree. A truncated tree that LOOKS complete to the
  consumer is worse than over-scrubbing leaves. Per
  silent-failure-hunter discipline + brief WP-3.
- **R4 (over-scrub bias):** Task 3 commentary documents the
  intentional permissive `email_regex` policy. Task 3 MUST NOT
  tighten the regex (deferred to v1+ per brief §8). Per GDPR §17 +
  ADR-015 the over-scrub bias is correct.
- **R5 (Task 0 audit):** enumerate ALL probes explicitly (per
  `.claude/rules/pre-phase-harness-audit.md`).
- **R6 (clippy invocations):** all clippy commands use
  `--workspace --features full --no-deps -- -D warnings` uniformly.
- **R7 (test-target compile gate):** Task 2 changes `scrub_json`
  internal shape but **public signature is unchanged**
  (`pub fn scrub_json(value: &Value) -> Value`). No re-export change.
  R7 (cargo test --no-run) is **non-invoked**. Verified by call-site
  audit: every caller passes a `&Value` and receives a `Value`; the
  helper additions (`scrub_json_inner` + `MAX_RECURSION_DEPTH`) are
  file-private. Re-verified by Task 0 Probe 8.
- **R8 (features full):** §15 cargo commands use
  `--workspace --features full`, NEVER `-p <crate> --features full`
  uniformly. **Exception**: the targeted unit-test invocation in §14
  uses `-p lemmy_db_schema --features full redaction::tests` because
  `lemmy_db_schema` defines a `full` feature
  (verified: `crates/db_schema/Cargo.toml` declares `[features] full`).
- **R9 (Junior worker e2e edit hang):** N/A — r1 makes **zero**
  e2e edits. The phase-tip e2e gate runs the existing 50+ e2e tests
  unchanged; existing redaction-relevant tests at e2e.rs lines 2444,
  2477, 2788, 2790, 2915, 2929, 5482 stay green.
- **R10 (call-site enumeration):** WP-6 audit run by planner at
  plan-author time (see §15.5); Task 0 Probe 6 re-verifies the
  count + the three gated-column write paths at lane-cut time
  per `feedback_fix_impl_enumerate_all_callsites.md`.

## 8. Flow design

### Before (governance-v0 HEAD pre-r1)

```
[caller: governance_log::append(payload)]
  → scrub_json(&payload)                   ← unbounded recursion
     → match Value:
        String → scrub(s)                  ← 3 replace_all (urls,mentions,email)
        Array  → recurse (no depth limit)
        Object → recurse (no depth limit)
        other  → clone

[caller: submit_jury_vote.rs:524/528 → public_case_log writes]
  → redaction::scrub(&summary)
  → redaction::scrub(&rationale)

[caller: admin_dashboard_html.rs (8 sites) → HTML render]
  → scrub() / scrub_json() at render time (defence-in-depth)

[caller: admin_rule_sets.rs:320 → API response]
  → scrub(&rsv.rule_text)

[tests: 5 happy-path unit tests at redaction.rs:109-158]
  → strips mentions, email, URLs, walks nested, preserves non-strings

[NO test for order-dependence (@bob@remote.example)]
[NO depth cap → stack overflow on adversarial 10K-deep tree]
[NO test for Unicode confusables]
[NO policy commentary on over-scrub bias]
```

### After (r1)

```
[caller: governance_log::append(payload)]               ← unchanged
  → scrub_json(&payload)
     → scrub_json_inner(&payload, 0)                    ← NEW indirection
        if depth >= MAX_RECURSION_DEPTH(64):
          return Value::Null                            ← HARD CAP (over-scrub bias)
        else:
          match Value: ... (same shape, depth+1 on recursion)

[caller: submit_jury_vote.rs / admin_dashboard_html.rs / admin_rule_sets.rs]
  → unchanged (same public scrub/scrub_json signatures)

[tests: 5 existing + 6 NEW active + 2 NEW #[ignore]'d at redaction.rs:tests]
  → existing 5 pass unchanged
  → new active: order-dependence, newline-mentions, regex-metacharacter,
                integer-preservation, Cyrillic-precedes-mention,
                depth-cap-returns-null
  → new #[ignore]: zero-width joiner, in-username Cyrillic confusable
                   (deferred to v1-redaction-r2; kind:"log" DQ carry-forward)

[commentary: NEW Maintenance invariants doc at module top]
  → order-dependence is load-bearing (test locks it)
  → scrub_json depth-cap is defence-in-depth (cap=64; rationale)
  → email_regex over-scrub bias intentional per GDPR §17 + ADR-015
```

**Box → task map:**
- Adversarial test corpus (6 active + 2 `#[ignore]`) → Task 1.
- `scrub_json` recursion bound + 1 depth-cap test → Task 2.
- Regex policy commentary + module-level Maintenance invariants → Task 3.

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first edit on each
task:

**Schema / type definitions (every task):**
- `crates/db_schema/src/source/governance/redaction.rs` (full file,
  159 lines) — the canonical implementation; doc-comments at lines
  1-25 (module + Phase-6 DQ-6.6 history note), 61-73 (scrub
  order-dependence), 82-87 (scrub_json invariants), 103-159 (existing
  test module).
- `crates/api/api/src/governance/redaction.rs` (22-line shim) — **DO
  NOT EDIT** per brief §2. Verifies no caller bypasses the canonical
  module.

**Existing patterns (per-task MIRROR refs — see §10):**
- Task 1: `redaction.rs:109-131` (the three single-regex tests —
  `scrub_strips_mentions`, `scrub_strips_email`, `scrub_strips_profile_urls`);
  `:134-148` (`scrub_json_walks_nested_structure`); `:150-158`
  (`scrub_preserves_keys_and_non_string_scalars`).
- Task 2: `redaction.rs:88-101` (current `scrub_json` shape).
- Task 3: `redaction.rs:1-25` (module doc); `:31-43` (mention_regex
  factory + existing boundary-class comment); `:45-51` (email_regex
  factory — currently un-commented); `:53-59` (profile_url_regex
  factory); `:61-73` (scrub order-dependence doc).

**Caller-side reads (planner-time WP-6 audit; impl re-verifies via
Task 0 Probe 6):**
- `crates/db_schema/src/source/governance/governance_log.rs:255-271`
  (`append` fn — the redaction chokepoint for `governance_log.payload`).
- `crates/api/api/src/governance/admin_dashboard_html.rs:18` (import
  line) + `:316-331` (8 render-time scrub call sites).
- `crates/api/api/src/governance/submit_jury_vote.rs:519-528`
  (the 2 redaction::scrub call sites for summary + rationale).
- `crates/api/api/src/governance/admin_rule_sets.rs:320`
  (`scrub(&rsv.rule_text)` in `RuleSetVersionView` map).
- `crates/apub/activities/src/governance/publish_sanction_notice.rs:291`
  (COMMENT only — intentional no-re-scrub per DQ-6.6 §2).
- `crates/apub/objects/src/protocol/governance/sanction_notice.rs:34`
  (DOC COMMENT only).
- `crates/server/tests/e2e.rs:2440-2929, 5482` (14 redaction-mentioning
  lines — existing e2e coverage; r1 makes ZERO e2e edits).

**Lessons (per `.claude/rules/advisor-orchestrator.md` §2.4 mandatory
file-class lesson injection — `redaction.rs` is NOT `e2e.rs` so the
e2e-edit-hang row doesn't apply; the test-module-add row applies via
the test-fn return-shape table):**
- `.claude/lessons/feedback_clippy_test_style.md` — Tasks 1+2 new
  unit tests use `assert_eq!` only; no `.unwrap()`/`.expect()`.
- `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md`
  — Tasks 1+2 mirror existing test pattern verbatim (plain `#[test]
  fn`, NOT `LemmyResult<()>`).
- `.claude/lessons/feedback_read_canonical_before_writing_spec.md` —
  every task Reads `redaction.rs:1-159` end-to-end BEFORE first edit.
- `.claude/lessons/feedback_features_full_workspace_only.md` +
  `.claude/lessons/feedback_features_full_p_crate_incompatible.md` —
  §15 commands use `--workspace --features full` uniformly; §14's
  targeted `-p lemmy_db_schema --features full` is the documented
  exception (the crate defines `full`).
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — every
  §13 task carries `creates:` + `modifies:` + `requires:` YAML.
- `.claude/lessons/feedback_laptop_default_for_validate_pending.md` —
  Tasks 1+2+3 push then write `kind: "validate-pending-laptop"` DQ;
  phase-tip e2e gate (post-Task 3) writes
  `kind: "validate-pending-laptop-e2e"` DQ.
- `.claude/lessons/feedback_validate_pending_laptop_must_use_wrapper.md`
  — laptop-side cargo invocations go through `.bat`/`.sh` wrapper.
- `.claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md` —
  phase-tip e2e gate uses `cmd //c "scripts\\brehon\\cargo-test.bat
  --workspace --test e2e --features full"`.
- `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` —
  WP-6 audit folded into §15.5 + §16a Story 4; Task 0 Probe 6 re-runs
  the `rg` enumeration to catch lane-cut-time drift.

## 10. Patterns to mirror

Per `feedback_advisor_watchpoint_specificity` discipline: every
pattern cites a specific file:line.

### 10.1 New unit test shape (Tasks 1 + 2)

**Mirror:** `crates/db_schema/src/source/governance/redaction.rs:109-131`
(the three existing single-regex tests).

```rust
#[test]
fn scrub_order_dependence_mention_with_remote_host_not_eaten_by_email() {
  // Locks docstring invariant at redaction.rs:66-73.
  // If someone re-orders the 3 replace_all calls in `scrub`, this fails:
  // mention regex must run before email regex so @bob@remote.example
  // is replaced as a whole unit, not as @[redacted] (orphan @).
  assert_eq!(
    scrub("hi @bob@remote.example see you"),
    "hi [redacted] see you"
  );
}
```

Plain `#[test] fn`. No `async`. No `LemmyResult`. `pretty_assertions::assert_eq`
already imported at line 106. New tests share the existing
`use super::*;` (line 105) and `use serde_json::json;` (line 107).

### 10.2 `#[ignore]` carry-forward test shape (Task 1 Unicode confusables)

**Mirror:** standard Rust `#[ignore]` attribute usage; no in-repo
sibling required.

```rust
#[test]
#[ignore = "v1-redaction-r2 will normalise via unicode-normalization crate"]
fn scrub_zero_width_joiner_between_at_and_handle() {
  // TODO(v1-redaction-r2): handle Unicode confusables (ZWJ injection
  // between @ and handle). v0 over-scrub bias is acceptable but ZWJ
  // adversarial vector is not currently caught. Tracked via DQ
  // kind:"log" carry-forward written at Task 1 completion.
  assert_eq!(
    scrub("@\u{200D}alice"),
    "[redacted]"
  );
}
```

The `#[ignore = "..."]` reason string surfaces in `cargo test`
output; the TODO comment names the deferral target sub-phase + the
mechanism (`unicode-normalization` crate).

### 10.3 Bounded recursion shape (Task 2)

**Mirror:** `crates/db_schema/src/source/governance/redaction.rs:88-101`
(current shape — replaced verbatim).

```rust
/// Maximum recursion depth for `scrub_json`. Defence-in-depth against
/// adversarial / buggy upstream JSON trees. Well below stack-overflow
/// threshold (~10K on x86_64) and generous for legitimate use
/// (governance_log payloads are flat — 5-6 levels worst case).
///
/// Hard cap; over-scrub bias under GDPR §17 + ADR-015 — a truncated
/// tree looking complete to the consumer is worse than visibly-null
/// leaves.
const MAX_RECURSION_DEPTH: usize = 64;

/// Recursively scrub every string value in a JSON tree.
///
/// Object **keys** are left intact because they are schema labels,
/// not user content. Values at every depth — strings, array elements,
/// object values — are passed through [`scrub`]. Non-string scalars
/// (numbers, booleans, nulls) are passed through unchanged.
///
/// Recursion bounded at [`MAX_RECURSION_DEPTH`] (64); deeper subtrees
/// substituted with [`Value::Null`] (over-scrub bias).
pub fn scrub_json(value: &Value) -> Value {
  scrub_json_inner(value, 0)
}

fn scrub_json_inner(value: &Value, depth: usize) -> Value {
  if depth >= MAX_RECURSION_DEPTH {
    return Value::Null;
  }
  match value {
    Value::String(s) => Value::String(scrub(s)),
    Value::Array(items) => Value::Array(
      items.iter().map(|v| scrub_json_inner(v, depth + 1)).collect(),
    ),
    Value::Object(map) => {
      let scrubbed = map
        .iter()
        .map(|(k, v)| (k.clone(), scrub_json_inner(v, depth + 1)))
        .collect();
      Value::Object(scrubbed)
    }
    other => other.clone(),
  }
}
```

Public signature `pub fn scrub_json(value: &Value) -> Value` is
**byte-identical to v0** — no caller updates, no re-export change.
`scrub_json_inner` is file-private (NO `pub` keyword);
`MAX_RECURSION_DEPTH` is a module-level `const` (file-private by
default).

The depth-cap test (paired with this change in the same task) walks
a deliberately-constructed 70-deep tree and asserts the leaf at
depth >= 64 is `Value::Null`. Test body:

```rust
#[test]
fn scrub_json_at_depth_cap_returns_null_not_truncated_tree() {
  // Locks the WP-3 invariant: at recursion depth >= MAX_RECURSION_DEPTH,
  // the substitution is Value::Null (NOT the un-scrubbed leaf, NOT a
  // truncated subtree). Over-scrub bias per ADR-015.
  let mut tree = Value::String("user @alice email foo@example.com".into());
  for _ in 0..70 {
    tree = Value::Array(vec![tree]);
  }
  let scrubbed = scrub_json(&tree);

  // Walk down 64 levels of the scrubbed tree. At each step, expect an
  // Array of length 1; at level 64 the inner value must be Value::Null
  // (NOT the original string, NOT a partial-scrub representation).
  let mut cursor = &scrubbed;
  for level in 0..64 {
    match cursor {
      Value::Array(items) => {
        assert_eq!(items.len(), 1, "level {level} should be Array(1)");
        cursor = &items[0];
      }
      other => panic!("level {level} expected Array, got {other:?}"),
    }
  }
  // At level 64 (the depth-cap boundary), the inner value is Null.
  assert_eq!(
    cursor,
    &Value::Null,
    "at depth 64, the substitution must be Value::Null (over-scrub bias)"
  );
}
```

### 10.4 Module-level commentary shape (Task 3)

**Mirror:** `crates/db_schema/src/source/governance/redaction.rs:1-25`
(current module doc + Phase-6 history note).

Insert a **new `## Maintenance invariants` H2 subsection** AFTER the
existing `## Crate placement (Phase 6, DQ-6.6 inbound resolution id
37)` section (currently ends at line 25), BEFORE the `use` block at
line 27. New text:

```rust
//! ## Maintenance invariants
//!
//! 1. **Order of operations in [`scrub`]** is load-bearing —
//!    profile URLs first, then fediverse mentions, then emails. See
//!    the `scrub` fn doc-comment below for the footgun this prevents.
//!    Locked by `scrub_order_dependence_mention_with_remote_host_not_eaten_by_email`
//!    in the test module.
//!
//! 2. **[`scrub_json`] recursion is bounded at
//!    [`MAX_RECURSION_DEPTH`] (= 64)** — defence-in-depth against
//!    adversarial / buggy upstream JSON trees. At the cap, the
//!    substitution is [`serde_json::Value::Null`] (NOT a truncated
//!    subtree). Over-scrub bias per GDPR §17 + ADR-015 — a leak
//!    defeats right-to-delete permanently; an over-scrubbed leaf is
//!    cosmetic loss.
//!
//! 3. **[`email_regex`] is permissive on purpose** — it accepts a
//!    superset of RFC-5322. False positives (e.g.
//!    `version-1.0@build-2026` scrubbed as email) are acceptable;
//!    false negatives are GDPR violations. Do NOT tighten without a
//!    new ADR amending ADR-015's over-scrub bias.
//!
//! 4. **Unicode confusables** (zero-width joiner, right-to-left
//!    override, Cyrillic lookalike in username) are partially covered
//!    by the ASCII-boundary-class behaviour but two known-deferred
//!    cases are tracked as `#[ignore]` tests in this module. Their
//!    resolution is owned by v1-redaction-r2 (not yet on roadmap).
```

Per-regex comments (Task 3 also adds):
- Above `email_regex` (currently between lines 44-45): one-line citation
  back to invariant 3 above.
- Above `mention_regex` (currently between lines 33-37 expands existing
  comment): one-line note that the ASCII boundary class is permissive
  for non-ASCII (boundary = `[^A-Za-z0-9._%+\-]` includes Unicode
  letters); citation back to invariant 4.

No production-code change. No regex tightening. No new test
(commentary-only edit; existing tests stay green).

## 11. Files to change

### `crates/db_schema` (1 crate, 1 file across all 3 impl tasks)

- `crates/db_schema/src/source/governance/redaction.rs` — **MODIFIED** —
  Task 1 appends 6 active + 2 `#[ignore]` tests in `#[cfg(test)] mod
  tests` (lines 103-159); Task 2 rewrites `scrub_json` (lines 88-101)
  + adds 1 depth-cap test; Task 3 inserts module-level "Maintenance
  invariants" subsection (between lines 25-27) + per-regex policy
  comments.

### NOT modified

- `crates/api/api/src/governance/redaction.rs` — 22-line `pub use`
  re-export shim; **DO NOT EDIT** per brief §2. Verified
  no signature drift in Task 2 (public `scrub_json` byte-identical).
- `crates/api/api/src/governance/admin_dashboard_html.rs` — 8 scrub
  call sites; **DO NOT EDIT** (call-site audit confirms correct
  defence-in-depth at render time; r1 has no requirement to change
  caller behaviour).
- `crates/api/api/src/governance/submit_jury_vote.rs:519-528` — 2
  redaction::scrub call sites; **DO NOT EDIT** (write-time
  scrubbing into `public_case_log.summary` +
  `.rationale_redacted`; correct).
- `crates/api/api/src/governance/admin_rule_sets.rs:320` — 1 scrub
  call site; **DO NOT EDIT** (read-time scrub of rule_text;
  defence-in-depth).
- `crates/db_schema/src/source/governance/governance_log.rs:268` —
  the redaction chokepoint (`payload: scrub_json(&payload)` inside
  `append`); **DO NOT EDIT** (call-site is correct; r1 doesn't widen
  the contract).
- `crates/server/tests/e2e.rs` — 14 redaction-mentioning lines;
  **DO NOT EDIT** (existing e2e coverage stays green; r1 adds NO
  e2e tests per brief §1).

### Struct-field add: none

r1 adds no fields to any struct (no `ReputationEventInsertForm`-class
change). No constructor-site enumeration required per
`feedback_planner_enumerate_struct_callsites_for_addfield.md`.

The new private fn `scrub_json_inner` (Task 2) is file-private;
`MAX_RECURSION_DEPTH` is a module-level `const`. Neither leaves the
file. Verified by call-site enumeration (§15.5 + §16a Story 4):
zero out-of-file references to `scrub_json_inner` or
`MAX_RECURSION_DEPTH`.

## 12. NOT building in v1-redaction-r1

- **`unicode-normalization` crate dependency** — deferred to
  v1-redaction-r2; brief §7 stop-and-ask tripwire #1. v1 ships the
  `#[ignore]` tests as in-tree TODOs + `kind: "log"` DQ
  carry-forward instead.
- **`email_regex` tightening to RFC-5322-strict form** — deferred to
  v1+ after pilot data shows real false-positive cost. Brief §7
  stop-and-ask tripwire #6 (no GDPR scope-expansion). Over-scrub bias
  is intentional per ADR-015 + GDPR §17.
- **IP address / phone number / HTML tag scrubbing** — out of scope.
  v0 explicit list per IMPLEMENTATION-PLAN-v0.md §4.2 is `@handle |
  email | profile URL`. Anything else is a new ADR.
- **Display-name blocklist (runtime-loaded from `person.name`)** —
  v1+ territory per IMPLEMENTATION-PLAN-v0.md §4.2; not on roadmap.
- **PII tokenization / hashing** — ADR-015 covers the pseudonym table
  at insert; redaction is a different mechanism (scrub-on-write to
  log columns).
- **Decryption / encryption of payloads** — out of scope; payloads
  are plain-JSON.
- **Re-redaction enforcement on read** — per `scrub_json` design
  (line 5 docstring), redaction is write-time only. Read-time is
  verbatim. This is a deliberate choice per GH #58
  ("`public_case_log.summary` read back verbatim").
- **`scrub_json` performance optimization** — current implementation
  allocates new `Map`/`Vec` on every walk. Acceptable for v0
  (governance_log writes are ~10/sec peak; scrub_json cost is
  microseconds). v1+ may revisit.
- **Edits to the api shim `crates/api/api/src/governance/redaction.rs`**
  — invariant per brief §2. The shim's `pub use` re-export of `scrub`
  + `scrub_json` from `lemmy_db_schema::source::governance::redaction`
  is structurally correct; editing it risks signature drift.
- **Edits to call sites under `crates/api/api/` or
  `crates/server/tests/`** — call-site audit (§15.5 + §16a Story 4)
  confirms zero bypass paths. r1 has no requirement to change caller
  behaviour. Any task proposing such an edit is a scope violation per
  brief §7.

---

## 13. Step-by-step tasks

> **Cohort dispatch:** Task 0 is a barrier (non-`[P]`). Tasks 1, 2, 3
> all edit the SAME file (`redaction.rs`) so **none of them are
> `[P]`** — they share `modifies:` paths and the cohort dispatch logic
> in `.claude/rules/advisor-orchestrator.md` §4.1 step 4 (YAML overlap
> check) refuses parallel dispatch when `modifies:` intersects.
> Sequential cohorts: T0 → T1 → T2 → T3 → phase-tip e2e gate → T4
> (retro).
>
> **Shape G:** SUSPENDED per
> `feedback_laptop_default_for_validate_pending.md`. r1 is a
> pre-Shape-G plan: cargo runs on laptop via the
> `validate-pending-laptop` handler. impl-task subagents on the
> EliteDesk write the `kind: "validate-pending-laptop"` DQ entry
> after push (Tasks 1+2+3); the phase-tip e2e gate writes
> `kind: "validate-pending-laptop-e2e"` for the e2e command. Every
> §15 + §14 cargo command goes through the `.bat`/`.sh` wrapper
> per `feedback_validate_pending_laptop_must_use_wrapper.md`.

### Task 0: Pre-flight harness audit + branch verification + WP-6 re-enumeration

**Goal:** verify environment is ready for v1-redaction-r1; confirm
branch is `phase-v1-redaction-r1`; confirm prior phase's `redaction.rs`
+ `governance_log.rs` deliverables are intact on the base; confirm
pre-existing clippy baseline is clean; confirm docker daemon up;
re-verify the WP-6 call-site count + bypass-path analysis at
lane-cut-time.

**FILES:**

```yaml
creates: []
modifies: []
requires: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5:
enumerate ALL probes explicitly):**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — wrapper sanity (cargo-check honors -p)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/PRPs/debug/v1-redaction-r1-audit-cargo-check-p.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-redaction-r1-audit-cargo-check-p.log
# EXPECT: exit 0; only lemmy_utils compiles

# Probe 2 — feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-redaction-r1-audit-cargo-check-features.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-redaction-r1-audit-cargo-check-features.log
# EXPECT: exit 0; workspace compiles with --features full

# Probe 3 — cargo-test wrapper honors target selection
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --no-run --features full > .claude/PRPs/debug/v1-redaction-r1-audit-cargo-test.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-redaction-r1-audit-cargo-test.log
# EXPECT: exit 0; e2e test target compiles workspace-wide

# Probe 4 — wrappers fail loud on cargo errors (exit-code propagation)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --no-run --features nonexistent_xyz > .claude/PRPs/debug/v1-redaction-r1-audit-cargo-test-negative.log 2>&1"
echo "cargo-test.bat exit on bogus feature: $?"
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features nonexistent_xyz > .claude/PRPs/debug/v1-redaction-r1-audit-cargo-check-negative.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"
# EXPECT: BOTH non-zero (typically 101)

# Probe 5 — clippy baseline against pre-r1 HEAD
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-redaction-r1-audit-clippy-baseline.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-redaction-r1-audit-clippy-baseline.log
# EXPECT: exit 0

# Probe 6 — WP-6 call-site re-enumeration (lane-cut-time drift check)
rg "scrub\\(|scrub_json\\(" crates/ --type rust | tee .claude/PRPs/debug/v1-redaction-r1-callsite-audit.log
rg "scrub\\(|scrub_json\\(" crates/ --type rust | wc -l
# EXPECT: count >= 21 (plan-author tip recorded 23); any new caller
# triggers a re-read of ±5 lines + a §16a Story 4 update if the new
# caller is on a write path into the three gated columns. If count
# differs from 23 by ≥2, file a kind:"blocker" DQ asking advisor to
# rerun the bypass-path analysis before any §13 impl task runs.

# Probe 7 — current branch is phase-v1-redaction-r1
git branch --show-current
# EXPECT: phase-v1-redaction-r1

# Probe 8 — canonical definition + shim invariant
wc -l crates/db_schema/src/source/governance/redaction.rs
# EXPECT: 159 (canonical)
wc -l crates/api/api/src/governance/redaction.rs
# EXPECT: 22 (shim, plan-author tip; tolerance ±2 — DO NOT EDIT per brief §2)
rg "^pub use lemmy_db_schema::source::governance::redaction::" crates/api/api/src/governance/redaction.rs
# EXPECT: 1 hit (the re-export line)

# Probe 9 — existing 5 unit tests are intact at the expected line range
rg -n "fn scrub_strips_mentions|fn scrub_strips_email|fn scrub_strips_profile_urls|fn scrub_json_walks_nested_structure|fn scrub_preserves_keys_and_non_string_scalars" crates/db_schema/src/source/governance/redaction.rs
# EXPECT: 5 hits (one per existing test)

# Probe 10 — pretty_assertions already in scope (no Cargo.toml change needed for new tests)
rg "pretty_assertions::assert_eq" crates/db_schema/src/source/governance/redaction.rs
# EXPECT: 1 hit (line 106)

# Probe 11 — lemmy_db_schema declares the `full` feature
rg '^\[features\]' crates/db_schema/Cargo.toml -A 5
# EXPECT: `full = ...` line present
rg '^full = ' crates/db_schema/Cargo.toml | head -1
# EXPECT: 1 hit (confirms -p lemmy_db_schema --features full is valid in §14)

# Probe 12 — concurrent-PR check (no other PR touches r1's IMPLEMENT file)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("governance/redaction\\.rs|governance/governance_log\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output; if any other lane is touching r1 files, STOP and file kind:"blocker" DQ
```

**EXPECT block:**
- Probes 0..3, 5, 7..12 exit 0 (or as documented per probe)
- Probe 4 exits NON-ZERO (negative test confirms exit-code propagation)
- Probe 6 returns count >= 21 (planner-time count was 23; drift of
  ±2 acceptable, > 2 triggers blocker DQ)
- Probe 7 returns `phase-v1-redaction-r1`
- Probe 8: 159 lines (canonical) + 22 lines (shim) + 1 re-export
- Probe 9: 5 existing test fns intact
- Probe 10: `pretty_assertions::assert_eq` already imported
- Probe 11: `lemmy_db_schema` defines `full` feature
- Probe 12: no concurrent PR overlap

**No commit at Task 0** — verification only. Any probe failure files
a `kind: "blocker"` DQ pending and stops.

### Task 1: Adversarial test corpus (WP-1, WP-2, WP-5, WP-7, WP-8)

**ACTION:** Append 6 active + 2 `#[ignore]` adversarial unit tests to
`#[cfg(test)] mod tests` in
`crates/db_schema/src/source/governance/redaction.rs`. Production code
(lines 1-101) is **unchanged** in this task.

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema/src/source/governance/redaction.rs   # append tests inside #[cfg(test)] mod tests
requires: []
```

**IMPLEMENT (file 1 of 1):** in
`crates/db_schema/src/source/governance/redaction.rs`, append the
following inside `mod tests { ... }` (existing module starts at line
103, currently ends at line 159 with `}`). Insert all new test fns
AFTER the existing `scrub_preserves_keys_and_non_string_scalars` test
(line 150-158), BEFORE the closing `}` at line 159.

Mirror the existing test shape verbatim per §10.1:

1. **`scrub_order_dependence_mention_with_remote_host_not_eaten_by_email`**
   — verbatim from §10.1 above. Locks WP-1 / H1.

2. **`scrub_handles_newline_separated_mentions`** (WP-7):

   ```rust
   #[test]
   fn scrub_handles_newline_separated_mentions() {
     assert_eq!(
       scrub("@alice\n@bob\n@carol"),
       "[redacted]\n[redacted]\n[redacted]"
     );
   }
   ```

3. **`scrub_does_not_treat_username_as_regex_pattern`** (WP-8):

   ```rust
   #[test]
   fn scrub_does_not_treat_username_as_regex_pattern() {
     // Username class is restrictive [A-Za-z0-9_-]+, so adversarial
     // patterns like @.*, @[abc], @(group) don't match — they're
     // literal text, not matched as mentions. Verifies the regex
     // engine isn't tricked into treating username content as a
     // pattern.
     assert_eq!(
       scrub("ok @abc bad @.* worse @[xyz]"),
       "ok [redacted] bad @.* worse @[xyz]"
     );
   }
   ```

4. **`scrub_json_preserves_integer_id_fields`** (WP-5):

   ```rust
   #[test]
   fn scrub_json_preserves_integer_id_fields() {
     // Confirms scrub_json passes integer scalars through unchanged
     // (issue #58 hint: "preserve schema-typed fields, scrub only
     // string identifiers"). Pseudonyms ARE strings but opaque-by-
     // construction; the scrubber regex doesn't match them.
     let input = json!({"community_id": 42, "actor_pseudonym": "abc123def"});
     let result = scrub_json(&input);
     assert_eq!(result["community_id"], json!(42));
     assert_eq!(
       result["actor_pseudonym"],
       json!("abc123def"),
       "pseudonym is opaque-by-construction; scrubber regex doesn't match"
     );
   }
   ```

5. **`scrub_mention_after_cyrillic_letter_is_scrubbed`** (WP-2 active):

   ```rust
   #[test]
   fn scrub_mention_after_cyrillic_letter_is_scrubbed() {
     // The ASCII boundary class [^A-Za-z0-9._%+\-] is permissive for
     // non-ASCII letters (they fall INTO the boundary set because
     // they're not in the excluded ASCII alphanumeric set). So a
     // mention preceded by a Cyrillic 'а' IS scrubbed.
     assert_eq!(
       scrub("hi а@alice"),
       "hi а[redacted]"
     );
   }
   ```

6. **`scrub_zero_width_joiner_between_at_and_handle`** (`#[ignore]` —
   WP-2 deferred) — verbatim from §10.2.

7. **`scrub_handle_with_unicode_confusable_in_username`** (`#[ignore]` —
   WP-2 deferred):

   ```rust
   #[test]
   #[ignore = "v1-redaction-r2 will normalise via unicode-normalization crate"]
   fn scrub_handle_with_unicode_confusable_in_username() {
     // TODO(v1-redaction-r2): Cyrillic 'а' inside the username slot
     // — current ASCII-only username pattern [A-Za-z0-9_\-]+ doesn't
     // match, so the mention is NOT scrubbed. Adversarial vector.
     assert_eq!(
       scrub("@аlice"),  // 'а' is Cyrillic U+0430
       "[redacted]"
     );
   }
   ```

**MIRROR:** `redaction.rs:109-131` (single-regex test pattern);
`:134-148` (json walk test pattern); `:150-158` (preserve-non-string
scalars test pattern).

**GOTCHA:**
- The existing test module imports `use super::*;` at line 105 and
  `use serde_json::json;` at line 107. Test 4 uses `json!(...)`
  without an additional import.
- `pretty_assertions::assert_eq` at line 106 supplies the `assert_eq!`
  macro; do NOT re-import `std::assert_eq` or the `assert_eq!`
  shadowing breaks.
- `#[ignore = "..."]` attribute reason string is Rust 1.41+; valid
  on the project's `rust-toolchain.toml` 1.95.
- WP-5 test: do NOT add fields named in a way that overlaps real
  payload shapes (e.g. don't use `case_id` because it might trigger
  later test-coupling). The synthetic `community_id: 42` is fine —
  it's a leaf integer, not a key the rest of the test suite cares
  about.
- The `а@alice` test (Cyrillic) uses U+0430 (NOT U+0061). The file
  is UTF-8 (`redaction.rs` already contains the em-dash `—` at line
  4); the Cyrillic char is also UTF-8 safe.

**VALIDATE (story-checkpoint feeds §16a Story 1):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-redaction-r1-task1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-redaction-r1-task1-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-redaction-r1-task1-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-redaction-r1-task1-clippy.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_db_schema --features full redaction::tests > .claude/PRPs/debug/v1-redaction-r1-task1-test.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-redaction-r1-task1-test.log
# EXPECT: exit 0; "test result: ok. 11 passed; 0 failed; 2 ignored"
# (5 existing + 6 new active + 2 #[ignore]'d = 13 total tests; 11 run, 2 ignored)
```

Per `feedback_laptop_default_for_validate_pending.md`: after push,
write `kind: "validate-pending-laptop"` DQ entry listing the three
commands above; advisor handles the laptop-side cargo run.

**DQ at task completion (kind: "log"):** write a `kind: "log"` DQ
entry recording the two `#[ignore]`'d Unicode confusable tests as
v1-redaction-r2 carry-forward. Schema per
`.claude/rules/decision-queue.md`:

```json
{
  "id": "<v3 composite>",
  "from": "impl",
  "kind": "log",
  "answered_by": "impl-self-resolved",
  "question": "Unicode confusables (ZWJ injection + in-username Cyrillic lookalike) deferred to v1-redaction-r2 per IMPLEMENTATION-PLAN-v0.md §4.2",
  "options": ["v1-redaction-r2", "v0+unicode-normalization"],
  "answer": "v1-redaction-r2 owns Unicode normalisation; v0 ships #[ignore]'d tests in redaction.rs as in-tree carry-forward markers."
}
```

### Task 2: Bounded `scrub_json` recursion (WP-3)

**ACTION:** Rewrite `scrub_json` in
`crates/db_schema/src/source/governance/redaction.rs` to introduce a
file-private `scrub_json_inner(value, depth)` helper + a module-level
`const MAX_RECURSION_DEPTH: usize = 64`. Public signature
`pub fn scrub_json(value: &Value) -> Value` is **byte-identical**.
At depth `>= MAX_RECURSION_DEPTH`, substitute `Value::Null` (over-
scrub bias per ADR-015 + GDPR §17). Add 1 paired depth-cap test.

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema/src/source/governance/redaction.rs   # rewrite scrub_json + add depth-cap test
requires:
  - task: 1
    reason: Task 1's adversarial tests share the `#[cfg(test)] mod tests` block with this task's depth-cap test; sequential to avoid commit-overlap on the same file.
```

**IMPLEMENT (file 1 of 1):**

(a) **Production code rewrite** (replace current lines 88-101): use
the verbatim shape in §10.3 above. The new code:
- Adds `const MAX_RECURSION_DEPTH: usize = 64;` at module scope
  (insert above the new `scrub_json` fn).
- Rewrites `pub fn scrub_json(value: &Value) -> Value` to a 1-line
  delegation to `scrub_json_inner(value, 0)`.
- Adds `fn scrub_json_inner(value: &Value, depth: usize) -> Value`
  with the depth-cap check at the top + recursive walk with
  `depth + 1`.
- Updates `scrub_json`'s doc-comment per §10.3 (mentions
  `MAX_RECURSION_DEPTH` + `Value::Null` substitution).

(b) **New test fn** (append inside `mod tests { ... }`, after Task 1's
new tests, before closing `}`): use the verbatim shape from §10.3
(`scrub_json_at_depth_cap_returns_null_not_truncated_tree`).

**MIRROR:** §10.3 (the rewrite shape — current code at
`redaction.rs:88-101` is the verbatim replacement target).

**GOTCHA:**
- `MAX_RECURSION_DEPTH = 64` is module-level (NOT inside a fn);
  insertion point is between the existing `profile_url_regex` fn
  (ends at line 59) and the `/// Strip identifiers from ...`
  doc-comment at line 61. The const + its own doc-comment sit
  above `scrub` so both `scrub` and `scrub_json` reference it.
- The depth check at the TOP of `scrub_json_inner` short-circuits
  BEFORE the `match`; this is the correct silent-failure refusal
  shape (NOT a `match` arm that falls through to a default).
- The depth-cap test walks 64 levels into the scrubbed tree. The
  test asserts `Array(1)` at each level; the inner cursor at level
  64 must be `Value::Null`. This is non-trivial — read the test
  body carefully; do NOT shortcut by asserting only the top-level
  shape.
- `pub fn scrub_json` signature is unchanged: same `&Value` input,
  same `Value` output. R7 is non-invoked. Re-verify by
  `rg "fn scrub_json" crates/db_schema/src/source/governance/redaction.rs`
  — should still return 1 hit (no `pub(crate)` widening).
- `scrub_json_inner` is the helper. Do NOT make it `pub` or
  `pub(super)`; the file-private default is correct. Verified by
  `rg "scrub_json_inner" crates/` post-edit — expect exactly 3 hits
  (definition + 2 recursive calls inside its own body).

**VALIDATE (story-checkpoint feeds §16a Story 2):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-redaction-r1-task2-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-redaction-r1-task2-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-redaction-r1-task2-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-redaction-r1-task2-clippy.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat -p lemmy_db_schema --features full redaction::tests > .claude/PRPs/debug/v1-redaction-r1-task2-test.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-redaction-r1-task2-test.log
# EXPECT: exit 0; "test result: ok. 12 passed; 0 failed; 2 ignored"
# (Task 1's 6 new active + this task's 1 new active + 5 existing = 12 active; +2 ignored)
```

Per `feedback_laptop_default_for_validate_pending.md`: after push,
write `kind: "validate-pending-laptop"` DQ.

### Task 3: Regex commentary + module-level Maintenance invariants (WP-4)

**ACTION:** Insert a new `## Maintenance invariants` H2 subsection
into the module-level doc-comment of
`crates/db_schema/src/source/governance/redaction.rs` (between the
existing `## Crate placement` section and the `use` block at line 27),
plus one-line policy citations above `email_regex` (line ~45) and
`mention_regex` (line ~33) per §10.4. **Comment-only edits; zero
behaviour change; zero new tests.**

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema/src/source/governance/redaction.rs   # comment-only insertions
requires:
  - task: 2
    reason: Task 3's Maintenance-invariant #2 references `MAX_RECURSION_DEPTH` introduced by Task 2; sequential on the same file.
```

**IMPLEMENT (file 1 of 1):** insert the §10.4 verbatim block as the
new module-level subsection. Then add per-regex one-line policy
citations:

- Above `email_regex` (currently between lines 44-45):

  ```rust
  // Permissive on purpose — see Maintenance invariants #3 in the
  // module doc above. Over-scrub bias per GDPR §17 + ADR-015.
  ```

- Above `mention_regex` (extending the existing comment block at
  lines 33-37; add ONE new line at the end of the existing comment,
  before the `#[expect(clippy::expect_used, ...)]` attribute):

  ```rust
  // ASCII boundary class — Unicode-permissive by side-effect; see
  // Maintenance invariants #4 in the module doc above.
  ```

**MIRROR:** §10.4 above (verbatim module-doc block);
`redaction.rs:1-25` (existing module-doc shape with `//!` prefix).

**GOTCHA:**
- All inserted text uses the `//!` prefix for the module-level block
  (NOT `///` — that would be an item doc-comment and rustdoc would
  complain about being attached to the wrong item).
- Per-regex citations use `//` (plain comments), NOT `///` (NOT
  doc-comments — `mention_regex` etc. are non-public fns; rustdoc
  ignores them anyway).
- Markdown headings inside `//!` use 4-space indent for code blocks
  per `rustdoc` rendering. The text in §10.4 is already correctly
  shaped for rustdoc; copy verbatim, do NOT reformat.
- `#![deny(clippy::doc_lazy_continuation)]` is workspace-level on
  this crate (verify in `crates/db_schema/src/lib.rs`); the new
  Maintenance invariants paragraphs use a blank `//!` line between
  list items and use a leading word other than "and" on continuation
  lines per `feedback_clippy_doc_lazy_continuation_in_doc_comments.md`
  (§10.4 text already complies — review at impl time before commit).

**VALIDATE (story-checkpoint feeds §16a Story 3):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-redaction-r1-task3-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-redaction-r1-task3-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-redaction-r1-task3-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-redaction-r1-task3-clippy.log
# EXPECT: exit 0 (doc_lazy_continuation specifically must not fire)
```

Per `feedback_laptop_default_for_validate_pending.md`: after push,
write `kind: "validate-pending-laptop"` DQ.

### Phase-tip e2e gate (post-Task 3, pre-PR)

Not a §13 task — runs once after Task 3 lands on phase branch. Per
brief §5 + `feedback_laptop_default_for_validate_pending.md`:

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-redaction-r1-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-redaction-r1-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-redaction-r1-e2e.log"
```

`run_in_background: true`; ~26 min on the laptop. The advisor writes
`kind: "validate-pending-laptop-e2e"` DQ before invoking. Pre-existing
50+ e2e tests must stay green — r1 makes ZERO e2e edits. Particularly:
the 7 redaction-mentioning e2e lines (2444, 2477, 2788, 2790, 2915,
2929, 5482) must pass unchanged.

### Task 4: Retro

**Goal:** author retro per `feedback_retro_not_report.md` and
`feedback_four_role_retro_signals.md`. One H2 per role
(Advisor / Planning / Impl / BM) with signals + lessons. Promote any
new lessons to `.claude/lessons/feedback_*.md` in the same retro
commit (per `feedback_one_system_memory_in_repo.md`).

Specifically reflect on:
- Did the planner-time WP-6 audit catch what it should have?
- Did the Task 0 Probe 6 re-verification catch any lane-cut-time drift?
- Did the `#[ignore]` + DQ carry-forward shape work for Unicode
  confusables, or is a different deferral mechanism needed for v1-r2?
- Did the over-scrub bias commentary (Task 3) feel sufficient, or did
  the impl session find itself wanting more cross-references?
- Cohort discipline: did keeping Tasks 1+2+3 sequential (all
  single-file) cost meaningful wallclock vs a parallel `[P]` cohort?
  (Expected answer: no — single-file edits cannot be `[P]`.)

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-redaction-r1-retro.md
modifies: []
requires:
  - task: 3
    reason: Retro reflects on the work-product of Tasks 0-3 + phase-tip e2e gate.
```

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cmd //c "scripts\\brehon\\cargo-check.bat
  --workspace --features full"` after every task (Tasks 1, 2, 3).
- **Lint:** `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace
  --features full --no-deps -- -D warnings"` after every task.
- **Targeted unit tests (per task):** `cmd //c
  "scripts\\brehon\\cargo-test.bat -p lemmy_db_schema --features full
  redaction::tests"`. The `-p lemmy_db_schema --features full` form is
  valid because `lemmy_db_schema` declares the `full` feature
  (verified by Task 0 Probe 11); this is the documented R8 exception
  for targeted fast-feedback testing during impl, NOT a substitute
  for the workspace gates above.
- **e2e execution (phase-tip, post-Task 3):** `cmd //c
  "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features
  full"` — the full e2e suite. Expected: existing 50+ tests pass; no
  new tests added by r1 (all r1 coverage lives as unit tests).
- **Migration round-trip:** N/A — r1 ships zero migrations.

---

## 15. Validation commands (DoD)

> **Planner-side discipline (per `feedback_plan_dod_dry_run_at_write.md`
> + `feedback_pre_phase_dod_smoke_test.md`):** every command in this
> section MUST be dry-run by the advisor against current HEAD before
> plan approval. Dry-run notes recorded in §19.

### 15.1 Static analysis (per task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-redaction-r1-task<N>-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-redaction-r1-task<N>-check.log
# EXPECT: exit 0
```

### 15.2 Lint (per task — uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-redaction-r1-task<N>-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-redaction-r1-task<N>-clippy.log
# EXPECT: exit 0
```

### 15.3 Test target compile

Not invoked per R7 — Task 2 makes no public-signature change; Tasks 1
and 3 are test/comment-only.

### 15.4 e2e test execution (phase-tip, post-Task 3)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-redaction-r1-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-redaction-r1-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-redaction-r1-e2e.log"
```

`run_in_background: true`; ~26 min on laptop. Per
`feedback_laptop_default_for_validate_pending.md` Shape G suspended;
advisor writes `kind: "validate-pending-laptop-e2e"` DQ before invoking.
EXPECT: `E2E_EXIT_0` appended; pre-existing 50+ e2e tests pass; no
new e2e tests required for r1.

### 15.5 Cross-cutting verification

Bulleted checklist of invariants the planner asserts hold at
end-of-phase:

- [ ] Every write into `governance_log.payload` goes through
      `scrub_json` (the only `append` fn at
      `crates/db_schema/src/source/governance/governance_log.rs:255-313`
      calls `scrub_json(&payload)` at line 268). **Audit confirmed at
      plan-author time; Task 0 Probe 6 re-verifies.**
- [ ] Every write into `public_case_log.summary` goes through
      `redaction::scrub` (the only emit at
      `crates/api/api/src/governance/submit_jury_vote.rs:524` calls
      `redaction::scrub(&build_summary(...))`). **Audit confirmed.**
- [ ] Every write into `public_case_log.rationale_redacted` goes
      through `redaction::scrub` (the only emit at
      `crates/api/api/src/governance/submit_jury_vote.rs:528` calls
      `redaction::scrub(&winning_rationales.join("\n"))`). **Audit
      confirmed.**
- [ ] Render-time defence-in-depth scrubbing intact (8 sites at
      `admin_dashboard_html.rs:316-331` + 1 site at
      `admin_rule_sets.rs:320`). **Audit confirmed; r1 does NOT
      edit these sites.**
- [ ] Existing 5 redaction unit tests pass unchanged
      (`scrub_strips_mentions`, `scrub_strips_email`,
      `scrub_strips_profile_urls`, `scrub_json_walks_nested_structure`,
      `scrub_preserves_keys_and_non_string_scalars`).
- [ ] New active tests pass: `scrub_order_dependence_*`,
      `scrub_handles_newline_separated_mentions`,
      `scrub_does_not_treat_username_as_regex_pattern`,
      `scrub_json_preserves_integer_id_fields`,
      `scrub_mention_after_cyrillic_letter_is_scrubbed`,
      `scrub_json_at_depth_cap_returns_null_not_truncated_tree`.
- [ ] New `#[ignore]` tests are file-present + carry the
      `v1-redaction-r2` reason string:
      `scrub_zero_width_joiner_between_at_and_handle`,
      `scrub_handle_with_unicode_confusable_in_username`.
- [ ] `kind: "log"` DQ entry exists (written by Task 1) naming
      v1-redaction-r2 as the Unicode-normalisation carry-forward owner.
- [ ] `scrub_json` public signature unchanged (`pub fn scrub_json(value:
      &Value) -> Value`); no caller updates required.
- [ ] `MAX_RECURSION_DEPTH` and `scrub_json_inner` are file-private;
      `rg "MAX_RECURSION_DEPTH|scrub_json_inner" crates/` returns
      hits only inside `redaction.rs`.
- [ ] No edit to `crates/api/api/src/governance/redaction.rs` (the
      22-line shim); `git diff phase-v1-redaction-r1 governance-v0 --
      crates/api/api/src/governance/redaction.rs` is empty.
- [ ] No edit to `crates/server/tests/e2e.rs`; `git diff
      phase-v1-redaction-r1 governance-v0 -- crates/server/tests/e2e.rs`
      is empty.
- [ ] No new migrations; `git diff phase-v1-redaction-r1 governance-v0
      -- migrations/` is empty.
- [ ] No new dependencies; `git diff phase-v1-redaction-r1
      governance-v0 -- Cargo.toml Cargo.lock crates/db_schema/Cargo.toml`
      is empty (or contains only whitespace).
- [ ] R1: new tests use `assert_eq!` only (no `.unwrap()` /
      `.expect()` / `dbg!`).
- [ ] R5: Task 0 enumerated all 12 probes.
- [ ] R6: all clippy invocations use `--no-deps` uniformly.
- [ ] R8: every workspace gate uses `--workspace --features full`;
      the targeted `-p lemmy_db_schema --features full redaction::tests`
      form appears only inside §14 / Task 1+2 VALIDATE blocks (fast
      feedback during impl, not a substitute for workspace gates).

### 15.6 DoD per workflow (Shape G plans)

Not applicable — Shape G SUSPENDED per
`feedback_laptop_default_for_validate_pending.md`. r1 is a pre-
Shape-G plan; cargo runs on laptop via `validate-pending-laptop` +
`validate-pending-laptop-e2e` DQ entries.

---

## 16. Acceptance criteria

Roll-up of §15 + §16a story checkpoints. The planner asserts each box
is ticked at end-of-phase. The advisor's `/brehon-verify` cross-checks
each box against the worktree branch before queueing `bm-merge`.

- [ ] Task 0 (pre-flight) all 12 probes pass
- [ ] Task 1 commit lands on `phase-v1-redaction-r1`
- [ ] Task 2 commit lands on `phase-v1-redaction-r1`
- [ ] Task 3 commit lands on `phase-v1-redaction-r1`
- [ ] §15.1 (cargo check) exit 0 after every task
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after
      every task
- [ ] §15.4 (phase-tip e2e) `E2E_EXIT_0` on the e2e log; pre-existing
      50+ tests pass; no new e2e tests added by r1
- [ ] §15.5 (cross-cutting verification) — all M boxes ticked
- [ ] §16a stories — all 4 stories `[done]`
- [ ] No edits to files outside §11 list (only
      `crates/db_schema/src/source/governance/redaction.rs` modified)
- [ ] Retro committed per §13 Task 4
- [ ] `kind: "log"` DQ written by Task 1 for the Unicode-confusable
      v1-redaction-r2 carry-forward
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo
      barrie-cork/lemmy`

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: Adversarial test corpus locks the docstring-asserted invariants (WP-1, WP-2 active, WP-5, WP-7, WP-8)

- **Composing tasks:** Task 1.
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat
  -p lemmy_db_schema --features full redaction::tests"`
- **Expected output:** `test result: ok. 11 passed; 0 failed; 2 ignored`
  (5 existing happy-path + 6 new active adversarial; 2 `#[ignore]`'d
  for v1-redaction-r2 carry-forward).
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `crates/db_schema/src/source/governance/redaction.rs` contains
    `fn scrub_order_dependence_mention_with_remote_host_not_eaten_by_email`
    declaration.
  - Same file contains `fn scrub_handles_newline_separated_mentions`,
    `fn scrub_does_not_treat_username_as_regex_pattern`,
    `fn scrub_json_preserves_integer_id_fields`,
    `fn scrub_mention_after_cyrillic_letter_is_scrubbed` declarations.
  - Same file contains two `#[ignore = "v1-redaction-r2 ...]`
    attributes (one above `scrub_zero_width_joiner_*`, one above
    `scrub_handle_with_unicode_confusable_in_username`).
  - `.claude/decision-queue.json` (on phase branch) contains a
    `kind: "log"` entry from `from: "impl"` answering
    `answered_by: "impl-self-resolved"` naming v1-redaction-r2 as
    the Unicode carry-forward owner.

### Story 2: Bounded scrub_json recursion prevents stack overflow on adversarial nesting (WP-3)

- **Composing tasks:** Task 2 (requires Task 1).
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat
  -p lemmy_db_schema --features full redaction::tests::scrub_json_at_depth_cap_returns_null_not_truncated_tree"`
- **Expected output:** `test result: ok. 1 passed; 0 failed`.
- **Brief-Scope outputs to verify:**
  - `crates/db_schema/src/source/governance/redaction.rs` contains
    `const MAX_RECURSION_DEPTH: usize = 64` at module scope.
  - Same file contains `fn scrub_json_inner` (file-private; no `pub`
    keyword).
  - `pub fn scrub_json(value: &Value) -> Value` signature unchanged
    (verified by `rg "pub fn scrub_json"
    crates/db_schema/src/source/governance/redaction.rs` returning 1
    hit at the same line range as v0).
  - `fn scrub_json_at_depth_cap_returns_null_not_truncated_tree`
    declaration present in test module.
  - No caller of `scrub_json` modified
    (`git diff phase-v1-redaction-r1 governance-v0 --
    crates/db_schema/src/source/governance/governance_log.rs
    crates/api/api/src/governance/admin_dashboard_html.rs
    crates/api/api/src/governance/submit_jury_vote.rs
    crates/api/api/src/governance/admin_rule_sets.rs` is empty).

### Story 3: Policy commentary documents the over-scrub bias for future maintainers (WP-4)

- **Composing tasks:** Task 3 (requires Task 2).
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-clippy.bat
  --workspace --features full --no-deps -- -D warnings"` — clippy
  pass confirms doc-comment integrity (no `doc_lazy_continuation`
  fires).
- **Expected output:** exit 0; clippy lints all pass; the new
  Maintenance-invariants doc block rustdoc-renders without warnings.
- **Brief-Scope outputs to verify:**
  - `crates/db_schema/src/source/governance/redaction.rs` contains
    a `//! ## Maintenance invariants` line (case-sensitive heading).
  - Same file contains a `//! ` line citing `ADR-015`.
  - Same file contains a `//! ` line citing `GDPR §17`.
  - Same file contains a `//! ` line citing `MAX_RECURSION_DEPTH`.
  - Same file contains a `//! ` line citing `v1-redaction-r2`.
  - Per-regex citations present above `mention_regex` and
    `email_regex` (one-line `//` plain comments referencing
    "Maintenance invariants" from the module doc).
  - **No production-code change** outside the comment blocks: `git
    diff phase-v1-redaction-r1~ phase-v1-redaction-r1 --
    crates/db_schema/src/source/governance/redaction.rs` shows
    only `//!` / `//` / blank-line additions in the Task 3 commit
    (no `fn`/`pub`/`const`/`use` token changes).

### Story 4: Call-site audit confirms zero bypass paths into the three GDPR-gated columns (WP-6)

- **Composing tasks:** done at plan-author time (recorded in §15.5);
  re-verified by Task 0 Probe 6 at lane-cut time.
- **Checkpoint command:** `bash -c 'rg "scrub\\(|scrub_json\\("
  crates/ --type rust > .claude/PRPs/debug/v1-redaction-r1-callsite-audit.log
  && wc -l < .claude/PRPs/debug/v1-redaction-r1-callsite-audit.log'`
- **Expected output:** count >= 21 (plan-author time was 23); each
  call site categorized as one of:
  - **Write path into one of the 3 gated columns** —
    `governance_log.rs:268`, `submit_jury_vote.rs:524`,
    `submit_jury_vote.rs:528`. (3 sites; chokepoints.)
  - **Render-time defence-in-depth** —
    `admin_dashboard_html.rs:316-331` (8 sites),
    `admin_rule_sets.rs:320` (1 site). (9 sites; correct.)
  - **Internal (definition + tests + self-references)** —
    `redaction.rs` (10 sites). (Correct.)
  - **Doc-comment only** —
    `publish_sanction_notice.rs:291` (1 site comment "we never
    re-scrub here"; intentional per DQ-6.6 §2),
    `sanction_notice.rs:34` (1 site doc-comment about callers
    requirement). (2 sites; documentation, not code paths.)
  Total: 3 + 9 + 10 + 2 = 24 (or 23, depending on rg counting
  doc-comment matches; verified at plan-author time = 23).
- **Brief-Scope outputs to verify:**
  - `.claude/PRPs/debug/v1-redaction-r1-callsite-audit.log` exists
    on phase branch + non-empty (committed by Task 0 — Probe 6
    `tee`s to this path).
  - The §15.5 cross-cutting verification block (above) has all 4
    "Every write into ..." boxes ticked.
  - `/brehon-verify` confirms zero bypass paths flagged in Task 0
    Probe 6's output (the file contents include the categorisation
    comment block above OR Task 0 Probe 6 emits the categorisation
    as a stdout summary the advisor records).

> **Verification mapping:** the advisor's `/brehon-verify` step (per
> `.claude/commands/brehon-verify.md`) iterates this section, runs
> each Story's Checkpoint against the worktree branch, and confirms
> each Brief-Scope output exists + matches its structural pattern.
> Phantoms (task complete but output absent or empty) trigger the
> catch-fire procedure in advisor-orchestrator.md.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all 12 probes confirmed)
- [ ] Task 1 (adversarial tests) committed
- [ ] Task 2 (recursion bound) committed
- [ ] Task 3 (policy commentary) committed
- [ ] §15 validation green at every gate
- [ ] §15.4 phase-tip e2e gate `E2E_EXIT_0`
- [ ] §16a stories all `[done]`
- [ ] Retro committed (Task 4)
- [ ] `kind: "log"` DQ written by Task 1 for v1-redaction-r2
      carry-forward
- [ ] PR opened by BM session against `governance-v0`
- [ ] CodeRabbit review complete with findings triaged per
      `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at
      `.claude/PRPs/reports/v1-redaction-r1-verify.md` shows all 4
      stories ✓
- [ ] Post-merge phase branch retained for retro reads

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| One of the adversarial tests in WP-1 / WP-2 / WP-5 / WP-7 / WP-8 FAILS against current v0 (the regex is broken in a way no one tested) | LOW | HIGH | Brief §7 stop-and-ask tripwire #4 fires — surface to user as a scope-expansion signal BEFORE authoring fix; do NOT silently fold into r1. The conformance audit at `.claude/PRPs/reports/conformance-audit-redaction-rs-2026-05-28.md` noted the existing regex behaviour matches the H1-H4 hypotheses, so this risk is residual. |
| The depth-cap value (64) is too low — real-world governance_log payloads exceed 64 levels and the cap fires under legitimate load | LOW | MED | Brief §7 stop-and-ask tripwire #3 — verify by reading the existing `governance_log` payload shapes (admin_dashboard_html.rs renders them; 5-6 levels worst case observed). If real shapes ever push past 64, the v1-redaction-r2 plan revises the cap; meantime the cap is conservative. |
| The depth-cap value (64) is too HIGH — adversarial payload at depth 60 still wastes stack | LOW | LOW | Stack frame for `scrub_json_inner` is ~80 bytes (PE on x86_64); 64 frames = ~5KB, well below Rust's default 8MB thread stack. Conservative. |
| A new call-site lands on `governance-v0` between plan-author time and lane-cut time, bypassing `scrub`/`scrub_json` for a write into the 3 gated columns | LOW | HIGH | Task 0 Probe 6 re-runs `rg` enumeration at lane-cut time. Drift of ±2 from the plan-author count (23) is acceptable; drift > 2 OR a new caller on a write path into the 3 columns files `kind: "blocker"` DQ and stops. |
| A future maintainer reads the over-scrub commentary and amends ADR-015 to tighten the email_regex | MED | HIGH | Out of scope for r1 to prevent (ADR amendments are a user-gated process per `.claude/rules/advisor-orchestrator.md` §3.2 gate 2). Task 3 commentary cites ADR-015 explicitly so any tightening attempt surfaces the dependency. |
| The Junior subagent paraphrases the §10.3 verbatim shape and introduces a subtle silent-failure (e.g. `depth > MAX_RECURSION_DEPTH` instead of `>=`) | LOW | HIGH | §10.3 + Task 2 IMPLEMENT block specify the shape verbatim; Story 2 checkpoint test (`scrub_json_at_depth_cap_returns_null_not_truncated_tree`) walks down 64 levels and asserts `Value::Null` at the cap boundary — a `>` vs `>=` off-by-one fails the test. |
| Lane B (v1-jm-b-followups-r1, per 2026-05-28 handover) races on `.claude/decision-queue.json` writes during the Task 1 `kind: "log"` DQ append | LOW | LOW | Multi-lane discipline per `.claude/rules/multi-lane-worktree.md` §"Layout" — Lane A and Lane B have separate worktrees with separate `.claude/decision-queue.json` paths. Daemon `.git/index.lock` cohort math: 2 concurrent Juniors (one per lane) below cohort-size-3 threshold per `feedback_cohort_shared_git_index_contention.md`. |
| `clippy::doc_lazy_continuation` fires on the new Maintenance invariants doc block | MED | LOW | §10.4 verbatim text formatted to avoid the lint (no mid-paragraph "and" continuations); Task 3 GOTCHA explicitly cites the lesson; Task 3 VALIDATE includes clippy check. |

---

## 19. Notes

### Plan-author-time DoD dry-run results

Per `feedback_plan_dod_dry_run_at_write.md`, the planner verified §15
commands against current HEAD on the lane-worktree:

- **§15.1 (cargo-check.bat / cargo-check.sh):** wrapper exists at
  `scripts/brehon/cargo-check.sh` (Linux daemon path) and
  `scripts/brehon/cargo-check.bat` (Windows laptop path); accepts
  `--workspace --features full`. Dry-run not invoked (planner is the
  daemon-side Junior; full cargo run is the laptop advisor's job per
  `feedback_laptop_default_for_validate_pending.md`). Wrapper-existence
  + flag-acceptance verified.
- **§15.2 (cargo-clippy.bat / .sh):** same — wrapper exists; accepts
  `--workspace --features full --no-deps -- -D warnings`.
- **§14 (targeted `-p lemmy_db_schema --features full redaction::tests`):**
  the `full` feature on `lemmy_db_schema` verified via
  `rg '^\[features\]' crates/db_schema/Cargo.toml -A 5` at plan-author
  time. **Result: feature `full` is defined.** Form is valid per the
  R8 exception.
- **§15.4 (e2e gate):** form `cmd //c "scripts\\brehon\\cargo-test.bat
  --workspace --test e2e --features full"` matches the canonical shape
  in `feedback_windows_e2e_requires_bat_wrapper.md`.
- **Task 0 Probe 6 (`rg`):** `rg "scrub\\(|scrub_json\\(" crates/
  --type rust | wc -l` returns `23` at the plan-author tip
  (`1edb8b94c`). Probe expectation set to "count >= 21" with drift
  alert at "> ±2" to absorb minor noise from doc-comment matches.

### Pre-queue clarify-DQ candidates (resolved at brief-author time)

The brief §10 listed 4 candidate clarify-DQs:

- **(a) WP-3 depth-cap value** — resolved by planner: `64`. Rationale
  per §18 risks table + §10.3 doc-comment.
- **(b) WP-4 over-scrub policy commentary placement** — resolved by
  planner: module-level Maintenance invariants subsection (§10.4) +
  per-regex one-line citations.
- **(c) WP-6 bypass-path remediation** — resolved by planner via
  audit at plan-author time: ZERO bypass paths detected (§15.5).
  Task 0 Probe 6 re-verifies at lane-cut time.
- **(d) WP-2 Unicode confusable test placement** — resolved by
  planner: in-tree `#[ignore]` tests (§10.2) + `kind: "log"` DQ
  carry-forward (Task 1 completion).

### Junior worker e2e edit hang (not invoked)

r1 makes ZERO edits to `crates/server/tests/e2e.rs`. The
`feedback_junior_worker_e2e_edit_hang` discipline applies only to
plans that DO edit e2e.rs in ≥2 Edits; r1 doesn't. The existing
50+ e2e tests (including the 7 redaction-mentioning lines) run
unchanged at the phase-tip gate.

### Existing test pattern is canonical for r1

Per `feedback_plan_stub_uniformity_with_canonical_sibling.md`: the
existing 5 redaction tests at `redaction.rs:109-158` use plain
`#[test] fn` (NOT `LemmyResult<()>`) because they're pure-function
tests with no async, no DB. The new adversarial tests mirror this
verbatim. The `feedback_lemmy_error_no_std_error` Case A/B/C analysis
does NOT apply because there are no `LemmyResult`-returning calls
in the test bodies (no `?` propagation at all — `assert_eq!` is the
sole verification mechanism).

### Brief §10 task-count interpretation

The brief §9 recommended 4-5 tasks including "T4 (call-site audit —
read-only)". The planner folded T4 into planner-time work
(§15.5 + §16a Story 4) + Task 0 Probe 6 re-verification, dropping
the Junior task. Rationale: T4 produces no code commit; its
deliverable is a categorisation that the planner must produce
anyway per `feedback_fix_impl_enumerate_all_callsites.md`; a
separate Junior task for `rg` + read-±5-lines burns context boot
for ~10s of mechanical work. Result: 3 impl tasks (T1/T2/T3) + T0
+ retro = 5 total tasks, matching the brief's "4-5 tasks" target.

### Lane mode

Per `.claude/rules/multi-lane-worktree.md` §"Lane modes": brief
declares **Mode B (mobile remote-control)**. The planning Junior
runs on the daemon; this plan file is committed on the worker
branch (`junior/...-planning-...`); the daemon's finalize-merge
brings it onto `governance-v0`. bm-cut then cuts
`phase-v1-redaction-r1` from `governance-v0`; subsequent impl-task
briefs follow Mode B (author on trunk + SSH-merge to phase branch
per the rule §"Brief location and trunk→phase sync").

### No new ENTRY_KIND consts; no new governance_log.rs touch

r1 is pure regex / recursion / commentary hardening on `redaction.rs`.
Zero ENTRY_KIND consts added — the registry at
`.claude/rules/governance-log-entry-kind-registry.md` is unchanged.
Brief §7 stop-and-ask tripwire fires if any §13 task proposes one.

---

## 20. Confidence score

- **Plan correctness:** 8/10 — the brief is unusually concrete (WP-1
  through WP-10 specify expected adversarial inputs + expected
  outputs verbatim; H1-H4 from the conformance audit pre-categorise
  the four hypothesis classes). Residual: one of the adversarial
  tests in §13 Task 1 could surface an actual regex bug (low
  probability per the conformance audit's framing).
- **Cargo budget:** 9/10 — three sequential single-file edits;
  ~6 GB peak per task; no migrations; no new dependencies. Laptop-
  side runs are well-characterised.
- **Test coverage:** 8/10 — 7 new active unit tests + 2 `#[ignore]`'d
  carry-forward + existing 5 unchanged = 14 total. e2e gate runs
  pre-existing 50+ unchanged. The depth-cap test walks down 64
  levels asserting `Value::Null`; the order-dependence test locks
  the docstring footgun verbatim; the Unicode-active test locks the
  ASCII-boundary-class permissive behaviour. The `#[ignore]`'d
  deferred tests are in-tree carry-forward markers, not coverage
  gaps disguised as passing tests.
