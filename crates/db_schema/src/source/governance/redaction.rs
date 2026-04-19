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
  email_regex().replace_all(&no_mentions, "[redacted]").into_owned()
}

/// Recursively scrub every string value in a JSON tree.
///
/// Object **keys** are left intact because they are schema labels, not
/// user content. Values at every depth — strings, array elements, object
/// values — are passed through [`scrub`]. Non-string scalars (numbers,
/// booleans, nulls) are passed through unchanged.
pub fn scrub_json(value: &Value) -> Value {
  match value {
    Value::String(s) => Value::String(scrub(s)),
    Value::Array(items) => Value::Array(items.iter().map(scrub_json).collect()),
    Value::Object(map) => {
      let scrubbed = map
        .iter()
        .map(|(k, v)| (k.clone(), scrub_json(v)))
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
    assert_eq!(scrub("hi @alice, see @bob@remote.example"), "hi [redacted], see [redacted]");
  }

  #[test]
  fn scrub_strips_email() {
    assert_eq!(scrub("contact foo.bar@example.com for details"), "contact [redacted] for details");
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
}
