# Planning Brief — v1-redaction-r1: Harden redaction regex (GDPR-critical, P0)

**Phase:** v1-redaction-r1
**Branch:** phase-v1-redaction-r1 (cut from `governance-v0` — exact SHA at lane-cut time)
**Authored:** 2026-05-28
**Authored by:** advisor (canonical brehon-fork session)
**Source issue:** https://github.com/barrie-cork/lemmy/issues/58 (P0 GDPR ship-blocker)
**PRD:** (none — narrow hardening pass; spec lives in `IMPLEMENTATION-PLAN-v0.md` §4.2 + ADR-015)
**Plan target:** `.claude/PRPs/plans/v1-redaction-r1.plan.md`

**Lane mode:** Mode B (mobile remote-control) per `.claude/rules/multi-lane-worktree.md` §"Lane modes". Brief authored on `governance-v0` in canonical; trunk→phase sync via daemon SSH per §"Brief location and trunk→phase sync".

**Pre-flight context:** conformance audit ran 2026-05-28 at brief-author time per advisor-orchestrator §3.1.1 — `redaction.rs` is in `crates/db_schema/src/source/governance/**`, in audit scope. Report at `.claude/PRPs/reports/conformance-audit-redaction-rs-2026-05-28.md`: **0 findings of any tier** (degenerate case — file is pure-Rust string + serde_json::Value transformation; the six-axis framework targets federation handler convention divergence, which `redaction.rs` is structurally outside). The audit DID surface four conformance-adjacent hypotheses (H1-H4) — those are the load-bearing planner inputs and appear in §3 + §4 below.

---

## 1. What this phase delivers

A targeted hardening pass on the v0 redaction layer to close ship-blocker gaps before pilot launch. Per [IMPLEMENTATION-PLAN-v0.md §4.2](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): "Leak of a username defeats right-to-delete permanently." Per ADR-015: the pseudonymised `actor_pseudonym` table is mandatory; the redaction layer is the implementation arm that guarantees raw identifiers never land in `public_case_log.summary`, `public_case_log.rationale_redacted`, or `governance_log.payload`.

Scope is **regex + recursion hardening + adversarial tests**, NOT a v1 redesign:

1. **Adversarial test corpus** — lock the existing v0 contract against known adversarial inputs. The current 5-test suite covers happy-path only. v0 ship needs at minimum: regex metacharacters in usernames, Unicode confusables (Cyrillic `а` lookalike, zero-width joiners), embedded newlines, mid-word `@handle` inside long-form reasoning text, deeply nested JSON, the docstring-asserted order-dependence footgun (`@bob@remote.example` → `[redacted]`, not `@[redacted]`).

2. **Regex hardening** — the boundary class on `mention_regex` is ASCII-only and the email regex is permissive. Tighten ONLY where over-scrub is acceptable; document the policy decision in module commentary.

3. **Recursion bound on `scrub_json`** — `scrub_json` is unbounded. Add a hard depth cap (proposed: 64) with an explicit `Value::Null` substitution at the cap, NOT a silent return (silent-failure refusal class). The depth cap is defence-in-depth — federation oversize cap at the deser layer should fire first per `feedback_lemmy_error_no_std_error.md` adjacent work.

4. **Schema-typed-field preservation** — issue #58 calls out: "preserve schema-typed fields, scrub only string identifiers." `scrub_json` currently scrubs every `Value::String`; integer fields like `community_id` are already untouched. But payload fields that hold IDs as strings (e.g. `actor_pseudonym: "abc123"`) MUST stay intact. Decision needed at plan-author time: is the existing behavior (scrub ALL string values) over-broad? Or is the issue's hint about preserving fields a design concern that doesn't apply because all pseudonyms ARE opaque by construction?

5. **`scrub` call-site audit** — `scrub` + `scrub_json` are called from at least 5 files (admin_dashboard_html.rs has 8 callsites alone; submit_jury_vote.rs, admin_rule_sets.rs, governance_log::append). Re-enumerate at plan-author time; ensure no caller bypasses `scrub`/`scrub_json` when handling user-supplied strings.

Ships as one PR — the four classes are mechanically interlinked (a recursion-bound change shifts test invariants, a regex tightening shifts adversarial expected outputs). Splitting would force adversarial-test re-baselining twice.

---

## 2. Key file anchors (verify these before authoring the plan)

| File | Anchor | Purpose |
|---|---|---|
| `crates/db_schema/src/source/governance/redaction.rs` | whole file (159 lines) | **CANONICAL DEFINITION** — was moved from `crates/api/api/src/governance/redaction.rs` during Phase 6 DQ-6.6-inbound (resolved id 37). Plan tasks edit THIS file. |
| `crates/api/api/src/governance/redaction.rs` | whole file (23 lines, `pub use` re-export shim) | Re-export shim — DO NOT EDIT. Kept stable so submit_jury_vote.rs:519 + admin_rule_sets.rs:320 + admin_dashboard_html.rs:18 continue to compile. |
| `crates/db_schema/src/source/governance/redaction.rs:31-43` | `mention_regex` | The ASCII-only boundary class (H3 — Unicode confusables) |
| `crates/db_schema/src/source/governance/redaction.rs:45-51` | `email_regex` | The permissive RFC-5322 superset (H4) |
| `crates/db_schema/src/source/governance/redaction.rs:53-59` | `profile_url_regex` | The URL form |
| `crates/db_schema/src/source/governance/redaction.rs:74-80` | `pub fn scrub` | The 3-call composition; **order-dependent** per docstring 66-73 (H1) |
| `crates/db_schema/src/source/governance/redaction.rs:88-101` | `pub fn scrub_json` | The unbounded recursive walker (H2) |
| `crates/db_schema/src/source/governance/redaction.rs:103-159` | `#[cfg(test)] mod tests` | 5 existing happy-path tests; insertion point for adversarial corpus |
| `crates/server/tests/e2e.rs` | lines 2444, 2477, 2788, 2790, 2915, 2929, 5482 (14 redaction-mentioning lines) | Existing e2e tests that exercise redaction via `governance_log::append`'s `scrub_json` layer. **Per `feedback_junior_worker_e2e_edit_hang.md` — if any e2e edits are needed, plan as multiple ≤200-line Edits, not one big Edit.** Most adversarial coverage should live as unit tests in `redaction.rs` `#[cfg(test)] mod tests`, NOT in e2e.rs. |
| `crates/api/api/src/governance/admin_dashboard_html.rs:18,316-331` | 8 `scrub`/`scrub_json` callsites | Call-site audit target — verify no bypass paths |
| `crates/api/api/src/governance/admin_rule_sets.rs:320` | `scrub(&rsv.rule_text)` | Call-site audit target |
| `crates/api/api/src/governance/submit_jury_vote.rs:519,523` | `redaction::scrub(...)` for summary + rationale | Call-site audit target |
| `crates/db_schema/src/source/governance/governance_log.rs` | `append` fn | Call-site audit target — verifies `scrub_json` is the only redaction path for payload |
| `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` | ADR-015 (pseudonym discipline) | Required reading — the spec the redaction layer implements |
| `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` | §4.2 (redaction explicit simplification) | Required reading — v0 scope boundary; "runtime-loaded blocklist" is v1, NOT v0 |

**Planner MUST re-enumerate at plan-author time:**

```bash
rg "scrub\\(|scrub_json\\(" crates/ --type rust
```

per `feedback_fix_impl_enumerate_all_callsites.md`. The current count is ~15 callsites across ~5 files. Drift between brief-time and lane-cut counts → file `kind: "blocker"` DQ before authoring.

---

## 3. Required reading (planner subagent MUST read before authoring the plan)

1. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` ADR-015 — pseudonym discipline (the spec).
2. `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` §4.2 — redaction explicit simplification (v0 scope boundary).
3. `crates/db_schema/src/source/governance/redaction.rs` (full file) — current implementation + docstring invariants.
4. `.claude/PRPs/reports/conformance-audit-redaction-rs-2026-05-28.md` — conformance audit report + four conformance-adjacent hypotheses (H1-H4).
5. `crates/api/api/src/governance/admin_dashboard_html.rs:18,316-331` — heaviest caller surface; planner reads to understand the JSONB flow.
6. `crates/db_schema/src/source/governance/governance_log.rs` — `append` fn, the redaction chokepoint for the audit log.
7. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A discipline for any new unit test in `redaction.rs` (existing tests at lines 109-158 use simple `#[test] fn` with `assert_eq!` — they DON'T use `LemmyResult<()>` because they're pure-function tests, no async, no DB. Planner MUST decide whether to keep that pattern for new adversarial tests or to switch — current pattern is correct per `feedback_clippy_test_style.md`).
8. `.claude/lessons/feedback_async_pool_test_pattern.md` — N/A for unit tests in `redaction.rs`, APPLIES if any e2e additions land in `crates/server/tests/e2e.rs`.
9. `.claude/lessons/feedback_clippy_test_style.md` — `#![deny(unwrap, expect)]` for any new test; use `assert_eq!` + `?` (not `.unwrap()`).
10. `.claude/lessons/feedback_postgres_jsonb_canonicalization.md` — N/A for the regex layer, but planner should be aware: the `scrub_json` output flows into PG JSONB columns; the canonicalization happens at PG side, not at scrub side. No action needed for v0.

---

## 4. Watchpoints for the planner

**WP-1 (H1 from audit — lock the order-dependence test).** The module doc at `redaction.rs:66-73` asserts a load-bearing ordering invariant:

> Order matters: profile URLs first, then fediverse mentions, then plain email addresses. If mentions were processed after email, a `@bob@remote.example` mention would first have its `bob@remote.example` tail eaten by the email regex, leaving an orphan `@[redacted]`.

No unit test asserts this. The existing test at line 134 uses `@bob` (no `@host` tail) — doesn't exercise the footgun. **Required adversarial test** (cite the docstring; assert the specific footgun):

```rust
#[test]
fn scrub_order_dependence_mention_with_remote_host_not_eaten_by_email() {
    // Locks docstring invariant at redaction.rs:66-73.
    // If someone re-orders the 3 replace_all calls in `scrub`, this fails.
    assert_eq!(
        scrub("hi @bob@remote.example see you"),
        "hi [redacted] see you"
    );
}
```

**WP-2 (H3 from audit — Unicode confusables + boundary class).** Adversarial: a username preceded by a Cyrillic letter, Greek letter, CJK char, or emoji. The current ASCII boundary class `[^A-Za-z0-9._%+\-]` works because non-ASCII letters fall INTO the "boundary" set (they're not in the excluded ASCII alphanumeric set), so `"hi а@alice"` (Cyrillic `а`) IS scrubbed. BUT:

- Zero-width joiner (`\u{200D}`) between letters could create lookalike adversarial patterns.
- `@` preceded by an ASCII letter inside a word (e.g. `"email me at info@example.com via foo@bar"`) is the email path — the mention regex's boundary class correctly defers to email.
- Right-to-left override character (`\u{202E}`) between `@` and handle.

Required adversarial tests:
```rust
#[test]
fn scrub_mention_after_cyrillic_letter_is_scrubbed() { /* "а@alice" */ }
#[test]
fn scrub_zero_width_joiner_between_at_and_handle() { /* "@\u{200D}alice" */ }
#[test]
fn scrub_handle_with_unicode_confusable_in_username() { /* "@аlice" — Cyrillic а */ }
```

**Decision point for planner:** does v0 add `unicode-normalization` crate to normalize input before scrubbing? Default: **NO** (v0 simplification is conservative regex-only per IMPLEMENTATION-PLAN-v0.md §4.2). Add the failing tests as `#[ignore]` with a `// TODO(v1-redaction-r2): handle Unicode confusables` comment AND a `kind: "log"` DQ entry recording the v1 carry-forward. The `#[ignore]` ensures the failing-case is documented in-tree as known-deferred, not silently absent.

**WP-3 (H2 from audit — bound scrub_json recursion).** The current `scrub_json` walks `Object` and `Array` recursively without depth guard. Stack overflow on a 10K-deep payload is theoretically possible. Proposed bound:

```rust
const MAX_RECURSION_DEPTH: usize = 64;

pub fn scrub_json(value: &Value) -> Value {
    scrub_json_inner(value, 0)
}

fn scrub_json_inner(value: &Value, depth: usize) -> Value {
    if depth >= MAX_RECURSION_DEPTH {
        return Value::Null;  // hard cap; over-scrub bias under GDPR
    }
    match value {
        Value::String(s) => Value::String(scrub(s)),
        Value::Array(items) => Value::Array(items.iter().map(|v| scrub_json_inner(v, depth + 1)).collect()),
        Value::Object(map) => Value::Object(
            map.iter().map(|(k, v)| (k.clone(), scrub_json_inner(v, depth + 1))).collect()
        ),
        other => other.clone(),
    }
}
```

**The substitution at the depth cap MUST be `Value::Null`, NOT silent return of the truncated tree.** Per silent-failure-hunter discipline: a "truncated at depth N" payload that LOOKS like a complete payload to the consumer (admin dashboard, audit log) is worse than over-scrubbing the leaves.

Required adversarial test:
```rust
#[test]
fn scrub_json_at_depth_cap_returns_null_not_truncated_tree() {
    // Build a 70-deep nested array of strings containing identifiers.
    // Assert the leaves at depth 64+ are Value::Null, NOT the un-scrubbed string.
    let mut tree = Value::String("user @alice email foo@example.com".into());
    for _ in 0..70 { tree = Value::Array(vec![tree]); }
    let scrubbed = scrub_json(&tree);
    // Walk down to depth 64 — that level should be Null.
    // ...
}
```

**Decision point for planner:** is 64 the right cap? Tradeoff: 64 is generous for legitimate use (governance_log payloads are flat; 5-6 levels in worst case). It's also well below stack-overflow threshold (~10K on typical x86_64). Plan §4.2 commentary cites the constant + rationale.

**WP-4 (H4 from audit — email regex permissiveness — policy decision).** The email regex `[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}` accepts non-RFC-5322 forms. False-positive scrubbing of strings like `version-1.0@build-2026` happens. Under GDPR, **over-scrub bias is correct** (GDPR §17 right-to-delete violation by under-scrub is far worse than over-scrub cosmetic loss). Required:

1. **Planner adds explicit policy commentary** to `redaction.rs` line ~50 (above `email_regex`): "Over-scrub bias is intentional per GDPR §17. False positives on non-email strings matching this pattern are acceptable; false negatives are GDPR violations." Cite ADR-015.
2. **No regex tightening for v0.** v1 may consider tighter RFC-5322 forms after pilot data shows actual false-positive cost.

**WP-5 (schema-typed field preservation — issue #58 hint).** Issue #58 calls out: "preserve schema-typed fields, scrub only string identifiers — e.g. scrubbing `community_id` integers breaks rollups." The current `scrub_json` already preserves non-strings (line 99-100 `other => other.clone()`). Required:

1. **Planner verifies via call-site audit** that no caller serializes integer IDs into strings before passing to `scrub_json`. Specifically check `governance_log::append`'s payload-construction sites for any `format!("{}", id)` patterns that would convert a typed integer to a string the scrubber then sees.
2. **Add adversarial test**: `scrub_json` preserves a `community_id: 42` integer; does NOT accidentally scrub `"community_id": "42"` (string form) — but if a caller is constructing the string form, the issue is at the caller, not at the scrubber.

```rust
#[test]
fn scrub_json_preserves_integer_id_fields() {
    let input = json!({"community_id": 42, "actor_pseudonym": "abc123def"});
    let result = scrub_json(&input);
    assert_eq!(result["community_id"], json!(42));
    assert_eq!(result["actor_pseudonym"], json!("abc123def"), "pseudonym is opaque-by-construction; scrubber regex doesn't match");
}
```

**WP-6 (call-site audit — no `scrub` bypass).** Planner runs `rg "scrub\\(|scrub_json\\(" crates/ --type rust` and produces a list. For each call site, planner reads ±5 lines of context and asserts: (a) the value flowing in is the OUTPUT of user-supplied content (e.g. a `summary` built from user-supplied rationale), and (b) NO sibling write to the same column bypasses `scrub`/`scrub_json`. Specifically check `governance_log::append` is THE only path into `governance_log.payload`. If any direct `INSERT INTO governance_log` bypasses `append`, file `kind: "blocker"` DQ — the bypass needs `scrub_json` added.

**WP-7 (embedded-newline + tab + other whitespace).** Adversarial: `"@alice\n@bob\n@carol"`. The regex `replace_all` should handle this (mention regex doesn't anchor to `\n` and the boundary class IS `\n`). Required adversarial test:
```rust
#[test]
fn scrub_handles_newline_separated_mentions() {
    assert_eq!(
        scrub("@alice\n@bob\n@carol"),
        "[redacted]\n[redacted]\n[redacted]"
    );
}
```

**WP-8 (regex metacharacter in username — safety).** `@` followed by regex metacharacters: `@.*`, `@[abc]`, `@\d+`. These should be scrubbed as literal sequences (the username pattern `[A-Za-z0-9_\-]+` does NOT match these chars, so the regex correctly fails to match). Required adversarial test:
```rust
#[test]
fn scrub_does_not_treat_username_as_regex_pattern() {
    // Adversarial: would a username `@evil` containing regex metacharacters
    // crash the engine? Username class is restrictive [A-Za-z0-9_-]+, so
    // patterns like @.*, @[abc], @(group) don't match (correct behavior:
    // they're literal text, not matched as mentions).
    assert_eq!(scrub("ok @abc bad @.* worse @[xyz]"), "ok [redacted] bad @.* worse @[xyz]");
}
```

**WP-9 (post-lane drift — RT-r3 / other lanes touching redaction call-sites).** This brief assumes the call-site count at the brief-author tip (`5d086e79b`). At lane-cut time, if any other lane has added `scrub` callsites, the call-site audit (WP-6) covers them. Planner re-runs `rg` enumeration at plan-author time per `feedback_fix_impl_enumerate_all_callsites.md`.

**WP-10 (ADR-006 advisory-only preservation — redaction doesn't enforce policy).** The redaction layer is a content-filter, not a policy gate. It MUST NOT make decisions about what content to drop, log, or warn on. It just transforms strings. If the planner finds a temptation to "scrub the whole payload if it looks suspicious" or similar — STOP. Per advisor-orchestrator §6 H2 (silent-failure refusal), redaction's contract is `String → String` and `Value → Value`; both are total functions.

---

## 5. DoD gates (cargo + e2e)

Per `feedback_plan_dod_dry_run_at_write.md`. **All DoD commands MUST be wrapper-prefixed (`cmd //c "scripts\\brehon\\cargo-<verb>.bat ..."`) per `feedback_validate_pending_laptop_must_use_wrapper.md` + `feedback_windows_e2e_requires_bat_wrapper.md`.**

**Task 1 (adversarial unit tests in `redaction.rs` — no behavior change):**
```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"
cmd //c "scripts\brehon\cargo-test.bat -p lemmy_db_schema --features full redaction::tests"
```
The `-p lemmy_db_schema` is valid here because `lemmy_db_schema` defines a `full` feature; verify at plan-author time per `feedback_features_full_p_crate_incompatible.md`.

**Task 2 (recursion bound on `scrub_json`):**
```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"
cmd //c "scripts\brehon\cargo-test.bat -p lemmy_db_schema --features full redaction::tests::scrub_json"
```

**Task 3 (regex commentary + policy notes — no behavior change):**
```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"
```

**Task 4 (call-site audit — read-only):**
```
rg "scrub\\(|scrub_json\\(" crates/ --type rust | tee .claude/PRPs/debug/v1-redaction-r1-callsite-audit.log
```
DoD: planner reviews the log, asserts each callsite is on a user-supplied-content path, files findings in plan §16a as `[done]`-state checkpoints.

**Phase-tip e2e gate (post-Task 4, pre-PR):**
```
cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-redaction-r1-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-redaction-r1-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-redaction-r1-e2e.log"
```
(run_in_background: true; ~26 min on the laptop). Per `feedback_laptop_default_for_validate_pending.md` — Shape G SUSPENDED through 2026-06-01.

**Existing 5 redaction unit tests MUST stay green** — backwards-compat check. No re-baselining of existing happy-path tests.

---

## 6. Lesson injections (mandatory, per advisor-orchestrator §2.4)

**Inject for all impl-tasks (file matches `crates/server/tests/e2e.rs` may not apply — most adversarial tests are unit tests in `redaction.rs`):**
- `feedback_lemmy_error_no_std_error.md` — N/A for `redaction.rs` unit tests (pure functions, no `LemmyResult` outer); APPLIES if any e2e edits land. **Note for planner:** the existing redaction tests at lines 109-158 use simple `#[test] fn` (no LemmyResult); new adversarial tests should mirror that pattern verbatim per `feedback_plan_stub_uniformity_with_canonical_sibling.md`.
- `feedback_clippy_test_style.md` — `#![deny(unwrap, expect)]`; new tests use `assert_eq!` + early returns; no `.unwrap()`.
- `feedback_fix_impl_enumerate_all_callsites.md` — call-site enumeration (WP-6) is a mandatory plan §13 task.

**Inject for Task 2 (recursion-bound change — touches `scrub_json` signature internally):**
- `feedback_multi_write_handlers_need_transactions.md` — analogue: the recursion-bound is a semantic-preserving refactor for the depth ≤ 64 case + a hard-cap at depth > 64. Document the semantic change in commit body.
- `pattern_test_against_reality_not_syntax.md` — verify the depth-cap test actually triggers (construct a tree 70 deep, walk down 64 levels, assert leaf is `Value::Null`).

**Inject for Task 4 (call-site audit):**
- `feedback_investigation_read_subsystem_end_to_end.md` — read each caller ±5 lines; don't just enumerate.
- `feedback_audit_trait_derive_validate_field_types.md` — analogue: a callsite that LOOKS like a `scrub` consumer might be transforming the value before passing (e.g. `format!("{}", scrub(s))` is fine; `format!("{}", s) // before scrubbing` is a bypass). Read.

**Inject for any e2e.rs edits (if planner decides to add e2e coverage beyond unit tests):**
- `feedback_junior_worker_e2e_edit_hang.md` — multiple ≤200-line Edits, not one big Edit.
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate verbatim `old_string`/`new_string` anchors at brief-author time if scope is fix-impl-ish (this brief is not fix-impl, but planner may invoke for impl-task brief authoring).
- `feedback_async_pool_test_pattern.md` — canonical pool/conn/LemmyResult fixture for any new e2e test.

**Inject for all impl-tasks:**
- `feedback_features_full_workspace_only.md` — every cargo gate uses `--workspace --features full`.
- `feedback_features_full_p_crate_incompatible.md` — `-p lemmy_db_schema --features full` is valid here BECAUSE `lemmy_db_schema` defines `full`. Verify at plan-author time.

---

## 7. Scope boundaries (stop-and-ask tripwires)

- **Stop if** the planner is tempted to add `unicode-normalization` crate dependency. v0 simplification per IMPLEMENTATION-PLAN-v0.md §4.2 is "conservative regex-only". File `#[ignore]` failing tests + `kind: "log"` DQ for v1 carry-forward instead.
- **Stop if** any call-site audit (WP-6) finds a bypass path INTO `public_case_log` or `governance_log.payload` that doesn't go through `scrub`/`scrub_json`. **This is the ship-blocker case** — surface to user immediately, do not silently fold into the plan.
- **Stop if** the recursion-bound number (proposed 64) needs to be different. The plan §4.2 commentary cites it; if real-world payloads push to depth >64, the cap is wrong — adjust before authoring.
- **Stop if** any adversarial test in WP-1 / WP-2 / WP-3 / WP-7 / WP-8 happens to FAIL against current v0 (i.e. the regex is broken in a way nobody tested). This is good news (the brief found a real bug) and bad news (scope expands beyond pure-hardening). Surface to user before authoring the plan.
- **Stop if** the conformance audit report (referenced in §3 #4) has been superseded or invalidated by a later audit. Verify `.claude/PRPs/reports/conformance-audit-redaction-rs-2026-05-28.md` exists at lane-cut time.
- **Stop if** RT-r3 (in-flight at brief-author time) has added scrub callsites that touch payload-shape changes. Re-run call-site enumeration.
- **Stop if** any GDPR scope-expansion is tempted (e.g. "scrub IP addresses too"). v0 scope is `@handle | email | profile URL`. Anything else is a new ADR.

---

## 8. Not in scope for v1-redaction-r1

- **Display-name blocklist (runtime-loaded from `person.name`).** Per IMPLEMENTATION-PLAN-v0.md §4.2: this is v1+ territory, NOT v0. Deferred to v1-redaction-r2 (not yet on roadmap).
- **Unicode normalization (NFKC).** Deferred. The H3 confusable tests land as `#[ignore]` + DQ carry-forward.
- **IP address scrubbing.** Out of scope. v0 explicit list is `@handle | email | profile URL`.
- **Phone number scrubbing.** Out of scope.
- **HTML tag stripping.** Out of scope (`scrub` operates on already-text strings; HTML escape is a different layer).
- **PII tokenization / hashing.** ADR-015 covers the pseudonym table at insert; redaction is a different mechanism (scrub-on-write to log columns).
- **Decryption / encryption of payloads.** Out of scope; payloads are plain-JSON.
- **Re-redaction enforcement on read.** Per `scrub_json` design (line 5 docstring), redaction is write-time only. Read-time is verbatim. This is a deliberate choice (see issue #58 comment about `public_case_log.summary` being "read back verbatim").
- **`scrub_json` performance optimization.** Current implementation allocates new `Map`/`Vec` on every walk. Acceptable for v0 (governance_log writes are ~10/sec peak; scrub_json cost is microseconds). v1+ may revisit.

---

## 9. Plan structure guidance

Per `.claude/PRPs/templates/plan.template.md`. Expected §13 task count: **4-5 tasks**.

| Task # | Deliverable | Files | `[P]`? |
|---|---|---|---|
| T0 | Pre-phase harness audit + clippy baseline + call-site enumeration | `.claude/PRPs/debug/v1-redaction-r1-task0-*.log` (diagnostic) | No |
| T1 | Adversarial test corpus (WP-1, WP-2, WP-5, WP-7, WP-8) — pure additions to `#[cfg(test)] mod tests`; existing 5 tests UNCHANGED | `crates/db_schema/src/source/governance/redaction.rs` (test module only) | Yes — file-disjoint with T2 (T2 edits production code; T1 edits test module) — but **deferred to plan-author judgment**: if the test module + production code share commit-overlap risk (single file edit, sequential cohort), set `[P]=No` to be safe |
| T2 | Recursion bound on `scrub_json` (WP-3) + adversarial test for depth-cap | `crates/db_schema/src/source/governance/redaction.rs` (production fn rewrite + 1 test) | No (same file as T1; sequential) |
| T3 | Regex commentary + policy notes (WP-4) — comment additions only | `crates/db_schema/src/source/governance/redaction.rs` (commentary) | No (same file) |
| T4 | Call-site audit (WP-6) — read-only enumeration; findings logged in plan §16a as `[done]` checkpoints | none (audit output to `.claude/PRPs/debug/`) | Yes — file-disjoint with all others |

**Cohort plan recommendation:**
- **Cohort 1:** T0 alone.
- **Cohort 2:** T1 + T4 dispatched in parallel `[P]` IF planner decides T1's test-module edits are file-disjoint enough from T4's read-only audit. Daemon `.git/index.lock` contention check per `feedback_cohort_shared_git_index_contention.md`: cohort size 2 is safe.
- **Cohort 3:** T2 alone (recursion bound — touches `scrub_json` production code).
- **Cohort 4:** T3 alone (commentary — same file).
- **Phase-tip gate:** e2e + Dependabot count (Dependabot N/A here — pure code change).

**Plan complexity:** 4/10. Estimated wall-clock: 1 day. T1+T2 carry the impl bulk (~2-3 hours impl + e2e gate ~26 min); T3 is ~30 min; T4 is ~1 hour read-only audit.

---

## 10. Pre-queue checklist (advisor to run before dispatching planning Junior)

- [ ] **Hard precondition check:** Lane B (v1-jm-b-followups-r1) is also being dispatched in parallel per the 2026-05-28 handover; verify daemon `.git/index.lock` cohort math holds (2 lanes × per-lane planner Junior = 2 concurrent workers, below the cohort-size-3 threshold per `feedback_cohort_shared_git_index_contention.md`).
- [ ] **RT-r3 conflict check:** `git diff origin/governance-v0..origin/phase-v1-RT-r3 -- crates/db_schema/src/source/governance/redaction.rs crates/api/api/src/governance/redaction.rs` — confirm empty. If non-empty, surface to user.
- [ ] **Call-site count re-verification:** `rg "scrub\\(|scrub_json\\(" crates/ --type rust | wc -l` — note count in brief commit body if it has drifted.
- [ ] **Conformance audit report exists:** `test -f .claude/PRPs/reports/conformance-audit-redaction-rs-2026-05-28.md`.
- [ ] `/brehon-clarify` run on this brief — candidate clarify-DQs: (a) WP-3 depth-cap value (64? 32? 128?), (b) WP-4 over-scrub policy commentary placement, (c) WP-6 bypass-path remediation (if any found — surface to user as ship-blocker), (d) WP-2 Unicode confusable test placement (in-tree `#[ignore]` vs. external v1 carry-forward only).
- [ ] `git -C C:/Users/barri/Developer/brehon-fork show governance-v0:.claude/PRPs/briefs/v1-redaction-r1-planning-1.md` — must succeed (brief committed before dispatch).
- [ ] `memory_search_hybrid("redaction regex GDPR pseudonym", limit: 5)` — check for lessons authored since 2026-05-25.
- [ ] `memory_search_hybrid("scrub_json recursion bound depth cap", limit: 5)` — check if depth-cap discipline has prior precedent in the lesson corpus.

---

## 11. Background: why this is P0 ship-blocker

Per `IMPLEMENTATION-PLAN-v0.md` §4.2: "Leak of a username defeats right-to-delete permanently." Per ADR-015: the pseudonymised `actor_pseudonym` table is mandatory; redaction is the implementation arm of pseudonym discipline.

Per GDPR §17 (right-to-erasure): a controller MUST be able to erase an individual's personal data on request. If usernames leak into `public_case_log.summary`, `public_case_log.rationale_redacted`, or `governance_log.payload` (the three columns the redaction layer is specifically designed to gate), and those columns are subsequently federated (Phase 6 outbound) or rendered in admin dashboards (admin_dashboard_html.rs), the leak is **structural and unrecoverable** — every consumer instance, every dashboard render, every audit-log query reproduces the leak.

The current v0 implementation (3 regexes + recursive `scrub_json`) is **conservatively correct for the happy path**. The hardening pass adds:

- Test-locked guarantees against the documented order-dependence footgun (no test exists today).
- Adversarial test coverage for Unicode confusables (currently untested — relevant given fediverse user-name patterns).
- A hard recursion bound on `scrub_json` (currently unbounded — stack-overflow theoretically possible on malicious payload).
- Policy commentary on the over-scrub bias (currently implicit — future maintainer might "fix" the email regex to be tighter, breaking GDPR compliance).
- A call-site audit confirming no bypass paths into the three gated columns.

None of these are speculative; all are the kind of latent ship-blocker that a real GDPR audit (during pilot launch) would surface as critical. Hardening before pilot is cheaper than incident response after pilot.
