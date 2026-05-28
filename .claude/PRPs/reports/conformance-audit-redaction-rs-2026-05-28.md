# Conformance audit — `crates/db_schema/src/source/governance/redaction.rs`

**Scope:** `file:crates/db_schema/src/source/governance/redaction.rs`
**Purpose:** brief-time (Lane A v1-redaction-r1 prevention check)
**Date:** 2026-05-28
**Auditor:** advisor session (canonical brehon-fork checkout, governance-v0)
**Mode:** read-only — no `Edit`/`Write` outside this report + `.claude/PRPs/audit-metrics/redaction-rs.json`

---

## Target symbols

| Symbol | Signature | LoC |
|---|---|---|
| `mention_regex` | `fn mention_regex() -> &'static Regex` | 8 |
| `email_regex` | `fn email_regex() -> &'static Regex` | 6 |
| `profile_url_regex` | `fn profile_url_regex() -> &'static Regex` | 6 |
| `scrub` | `pub fn scrub(text: &str) -> String` | 7 |
| `scrub_json` | `pub fn scrub_json(value: &Value) -> Value` | 14 |
| `tests` mod | `#[cfg(test)] mod tests` | 56 |

**Total:** 159 lines, 5 functions (3 regex factories + 2 public scrub fns), 5 unit tests.

## Six-axis findings

The six-axis framework is designed for federation handler convention divergence (conn-type,
append reborrow, trait-bound, error idiom, conn acquisition, ADR-015 pseudonym). `redaction.rs`
is a **degenerate case** for this framework: it has zero DB connection use, zero async, zero
trust-boundary error handling, zero actor-URL handling. It is pure-Rust regex + tree-walk
over `&str` + `serde_json::Value`.

| Axis | Finding | Tier | Sibling proof |
|---|---|---|---|
| 1. Conn-type / tx-boundary | **N/A** — no DB conn used | — | No sibling in `crates/db_schema/src/source/governance/` has a `scrub`-shape signature; all siblings are Diesel model wrappers with conn-taking `pub async fn` |
| 2. Append reborrow shape | **N/A** — no `governance_log::append` call | — | — |
| 3. Trait-bound completeness | **N/A** — no trait bounds on public fns; `scrub(&str) -> String` and `scrub_json(&Value) -> Value` are concrete | — | — |
| 4. Error idiom at trust boundary | **N/A** — no trust boundary; redaction is downstream of any verified-payload arrival, runs on already-trusted-as-belonging-to-this-instance text | — | — |
| 5. Conn acquisition idiom | **N/A** — no conn acquired | — | — |
| 6. ADR-015 pseudonym handling | **Tier 3 — stylistic note (NOT a finding)**: redaction is the GDPR scrubber that ADR-015 cites as the implementation arm of the pseudonym discipline. Module doc explicitly cites ADR-015 §4.2. No divergence to flag. | — | Module doc lines 6-13 + history note 17-25 |

**Tier 1 findings:** 0
**Tier 2 findings:** 0
**Tier 3 findings:** 0

## Hypotheses for the Lane A planner brief (NOT enforced — for §3 Required reading)

These are conformance-framework-adjacent risks the planner brief should surface, even though
the six axes don't directly target them:

### H1 — Regex order-dependence is asserted by docstring but not by test

The module doc at line 66-73 asserts a load-bearing ordering invariant:

> Order matters: profile URLs first (they contain `/u/<handle>` which must be replaced as a
> whole unit), then fediverse mentions (they have the leading `@` guard that distinguishes
> them from the email pattern's trailing `@host`), and finally plain email addresses. If
> mentions were processed after email, a `@bob@remote.example` mention would first have its
> `bob@remote.example` tail eaten by the email regex, leaving an orphan `@[redacted]`.

The unit tests at lines 110-131 test each regex in isolation. There is no unit test that
asserts the cited footgun (`@bob@remote.example` → `[redacted]`, not `@[redacted]`). The
single existing test that exercises mixed mentions (`scrub_json_walks_nested_structure` line
134) uses `@bob` (no `@host` tail), which doesn't exercise the order-dependence.

**Hypothesis:** if someone re-orders `scrub`'s three `replace_all` calls during a refactor, the
existing test suite passes but the docstring invariant breaks. **Suggested action for Lane A
planner brief §3:** add an explicit ordering test using `@bob@remote.example` to lock in the
docstring contract.

### H2 — `scrub_json` recursion depth is unbounded

`scrub_json` walks `Value::Object` and `Value::Array` recursively without depth guard. A
malicious or buggy upstream that constructs a deeply nested JSON tree (e.g. 10K-deep) would
blow the stack.

**Sibling pattern:** none in this module (`scrub_json` is the only recursive walker).
External `serde_json::Value` deserialization itself has a default recursion limit of 128 via
`serde_json::de::Deserializer::recursion_limit`, but `scrub_json` operates on an
already-deserialised `Value` — that protection has already fired.

**Hypothesis:** in v0, the inbound paths that call `scrub_json` (governance_log::append +
public_case_log writers) accept payloads bounded by the federation-inbound oversize cap
(per `feedback_lemmy_error_no_std_error.md` adjacent fix-impl-1 work, the oversize cap
fires at the deser layer, well before `scrub_json`). v1 adds tighter caps. Stack overflow
risk in practice is low but not zero.

**Suggested action for Lane A planner brief §3:** decide whether v1 redaction-hardening
includes an explicit recursion guard. If yes, planner brief lists it as an in-scope task; if
no, planner brief lists it as an out-of-scope OQ (deferred to v2).

### H3 — `mention_regex` boundary class is incomplete for Unicode

The `mention_regex` at line 40 uses `[^A-Za-z0-9._%+\-]` as the leading boundary class. This
is ASCII-only. A mention preceded by a Unicode letter (e.g. Cyrillic `а`, Greek `ω`, CJK,
emoji) would match as a "non-boundary" character — meaning the leading group would NOT match
and the `@` would not be scrubbed.

Example: `"привет @alice"` — the space is the boundary; OK.
Example: `"hi а@alice"` — the Cyrillic `а` is treated as "non-boundary" by the ASCII class;
the `@` IS scrubbed because the Cyrillic char is in the boundary class `[^A-Za-z0-9._%+\-]`.

Actually re-reading: the boundary class is `[^A-Za-z0-9._%+\-]` (negated ASCII alphanumeric +
URL-safe), so Cyrillic letters MATCH the boundary class (they're not in the negated set). So
Cyrillic-preceded mentions ARE scrubbed. **But** confusables / mid-word combining marks /
zero-width joiners might cause edge-case mismatches.

**Hypothesis:** Unicode confusable handling is a known adversarial vector against
ASCII-regex-based scrubbers. v1 redaction-hardening should consider whether to normalise
input via `unicode-normalization` crate before scrubbing.

**Suggested action for Lane A planner brief §3:** include adversarial Unicode confusable test
cases (Cyrillic `а` vs ASCII `a` lookalike usernames, zero-width joiner injection between
`@` and handle, etc).

### H4 — `email_regex` is permissive — accepts non-RFC-5322 forms

The email regex at line 49 is `[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}`. This
matches a broad superset of valid emails. False-positive risk is low for a scrubber (over-
scrubbing is safer than under-scrubbing under GDPR), but the issue body's "preserve schema-
typed fields, scrub only string identifiers" hint matters: if a non-email string happens to
match (e.g. `version-1.0@build-2026`), it gets scrubbed.

**Suggested action for Lane A planner brief §3:** explicit policy decision in planner OQ —
prefer over-scrub bias (current behavior) or tighten regex. Document the decision in plan
§4.2 commentary so future re-tightening doesn't drift.

## Conclusion

The six-axis conformance framework returns **0 findings of any tier** on this file because the
file is architecturally outside the framework's defect class. The framework targets federation
handler convention drift; `redaction.rs` is pure-Rust string/JSON transformation.

The four hypotheses (H1-H4) ABOVE are conformance-framework-adjacent and the Lane A planner
brief should surface them in §3 Required reading + §4 Watchpoints. The planner makes the
final call on whether each becomes an in-scope task, an out-of-scope OQ, or a deferred v2 item.

**Brief-author next action:** include this report's path in Lane A planning brief §3 Required
reading; include H1-H4 in §4 Watchpoints.
