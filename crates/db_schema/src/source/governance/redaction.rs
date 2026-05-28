//! Identifier scrubber for Brehon governance surfaces.
//!
//! Every string that lands in `public_case_log.summary`,
//! `public_case_log.rationale_redacted`, or the `governance_log.payload`
//! JSONB column is passed through [`scrub`] or [`scrub_json`] first.
//! Direct persistence of unredacted user strings into those columns is a
//! GDPR violation per [ADR-015] and a privacy leak per
//! [IMPLEMENTATION-PLAN-v0.md §4.2].
//!
//! v0 implements a conservative regex-only scrub per the plan §4.2
//! explicit simplification. A runtime-loaded display-name blocklist is
//! v1 territory (§4.2 specifies "startup-loaded blocklist from
//! `person.name`" as a v1 enhancement, not v0 scope).
//!
//! ## Crate placement (Phase 6, DQ-6.6 inbound resolution id 37)
//!
//! Originally lived in `crates/api/api/src/governance/redaction.rs`.
//! Moved to `lemmy_db_schema` so [`scrub_json`] can be called by
//! [`super::governance_log::append`] (also moved here), which in turn is
//! called by `crates/apub/activities/src/governance/inbox.rs` — that
//! crate cannot depend on `lemmy_api` because `Activity::receive` is
//! invoked in `lemmy_apub_activities` (lower in the dep graph than
//! `lemmy_api`). The previous module path `lemmy_api::governance::redaction`
//! is preserved as a `pub use` re-export in the api crate so the 2 existing
//! call sites in `submit_jury_vote.rs` continue to compile unchanged.

use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;

fn mention_regex() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  // The leading group anchors the mention to a non-identifier boundary
  // (start of string or a non-alphanumeric-ish character). Without it,
  // the `@` in an email like `foo.bar@example.com` would match as a
  // fediverse mention. `regex` has no lookbehind, so we capture the
  // boundary char and re-emit it in the replacement.
  #[expect(clippy::expect_used, reason = "static regex — infallible at startup")]
  RE.get_or_init(|| {
    Regex::new(r"(^|[^A-Za-z0-9._%+\-])@[A-Za-z0-9_\-]+(?:@[A-Za-z0-9._\-]+)?")
      .expect("valid regex")
  })
}

fn email_regex() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  #[expect(clippy::expect_used, reason = "static regex — infallible at startup")]
  RE.get_or_init(|| {
    Regex::new(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}").expect("valid regex")
  })
}

fn profile_url_regex() -> &'static Regex {
  static RE: OnceLock<Regex> = OnceLock::new();
  #[expect(clippy::expect_used, reason = "static regex — infallible at startup")]
  RE.get_or_init(|| {
    Regex::new(r"https?://[^\s/]+/(?:u|user|profile)/[A-Za-z0-9_\-]+").expect("valid regex")
  })
}

/// Maximum recursion depth for `scrub_json`. Defence-in-depth against
/// adversarial / buggy upstream JSON trees. Well below stack-overflow
/// threshold (~10K on x86_64) and generous for legitimate use
/// (governance_log payloads are flat — 5-6 levels worst case).
///
/// Hard cap; over-scrub bias under GDPR §17 + ADR-015 — a truncated
/// tree looking complete to the consumer is worse than visibly-null
/// leaves.
const MAX_RECURSION_DEPTH: usize = 64;

/// Strip identifiers from a human-readable string.
///
/// Replaces fediverse mentions (`@user`, `@user@host`), email addresses,
/// and profile URLs (`https://example.com/u/<handle>`,
/// `/user/<handle>`, `/profile/<handle>`) with the placeholder `[redacted]`.
///
/// Order matters: profile URLs first (they contain `/u/<handle>` which
/// must be replaced as a whole unit), then fediverse mentions (they have
/// the leading `@` guard that distinguishes them from the email
/// pattern's trailing `@host`), and finally plain email addresses. If
/// mentions were processed after email, a `@bob@remote.example` mention
/// would first have its `bob@remote.example` tail eaten by the email
/// regex, leaving an orphan `@[redacted]`.
pub fn scrub(text: &str) -> String {
  let no_urls = profile_url_regex().replace_all(text, "[redacted]");
  let no_mentions = mention_regex().replace_all(&no_urls, "$1[redacted]");
  email_regex()
    .replace_all(&no_mentions, "[redacted]")
    .into_owned()
}

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

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  #[test]
  fn scrub_strips_mentions() {
    assert_eq!(
      scrub("hi @alice, see @bob@remote.example"),
      "hi [redacted], see [redacted]"
    );
  }

  #[test]
  fn scrub_strips_email() {
    assert_eq!(
      scrub("contact foo.bar@example.com for details"),
      "contact [redacted] for details"
    );
  }

  #[test]
  fn scrub_strips_profile_urls() {
    assert_eq!(
      scrub("from https://lemmy.example/u/alice and https://other.example/user/bob"),
      "from [redacted] and [redacted]",
    );
  }

  #[test]
  fn scrub_json_walks_nested_structure() {
    let input = json!({
      "kind": "report_created",
      "note": "reported by @alice",
      "tags": ["foo", "@bob"],
      "nested": { "who": "carol@example.com", "count": 3 },
    });
    let expected = json!({
      "kind": "report_created",
      "note": "reported by [redacted]",
      "tags": ["foo", "[redacted]"],
      "nested": { "who": "[redacted]", "count": 3 },
    });
    assert_eq!(scrub_json(&input), expected);
  }

  #[test]
  fn scrub_preserves_keys_and_non_string_scalars() {
    let input = json!({
      "@handle_key": 42,
      "flag": true,
      "missing": null,
    });
    assert_eq!(scrub_json(&input), input);
  }

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

  #[test]
  fn scrub_handles_newline_separated_mentions() {
    assert_eq!(
      scrub("@alice\n@bob\n@carol"),
      "[redacted]\n[redacted]\n[redacted]"
    );
  }

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
}
