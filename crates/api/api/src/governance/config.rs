//! Brehon governance config reader.
//!
//! Single source of tuneable values for the whole governance platform. Every
//! hardcoded constant from Phase 4 (jury panel size, reputation deltas,
//! thresholds, founder multipliers, etc.) migrates to a row in
//! `governance_config` over Phase 5a/5b. Handlers never read constants
//! directly; they call `get_int`/`get_float`/`get_bool`/`get_text` with a
//! `Scope` and a dotted key.
//!
//! ## Fallback cascade
//!
//! For each read, the reader tries in order:
//!
//! 1. `community:<id>` row (most specific — v1 feature; no v0 handler writes
//!    community-scoped rows but the reader supports them for forward compat)
//! 2. `instance` row
//! 3. Rust `const DEFAULT_*` fallback defined below — Watch 1 parity contract
//!
//! The const fallback matters: if an admin accidentally deletes a seeded
//! row, reads degrade gracefully to the default instead of surfacing an
//! error.
//!
//! ## Parity contract (Watch 1)
//!
//! Every seeded row in the task-50 migration MUST have a matching Rust
//! `const` declared below, with a matching type. Two tests enforce this:
//!
//! - `parity::seeded_keys_count_matches_const_count` — structural, in-crate,
//!   no DB required.
//! - `config_parity_round_trip` — DB-backed; lives in
//!   `crates/server/tests/e2e.rs` per GOTCHA-50h landing-spot A. Calls the
//!   typed accessor for every seeded key with its declared `value_type`.
//!
//! ## ConfigCache
//!
//! Per-request memoisation. Constructed at handler entry
//! (`ConfigCache::new()`), threaded into the tx closure if the handler uses
//! `run_transaction`. First read per `(scope, key)` hits the DB; subsequent
//! reads return the cached value. The cache does NOT escape the request
//! — a new request starts fresh.

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::CommunityId;
use lemmy_db_schema_file::enums::MembershipState;
use lemmy_db_schema_file::schema::governance_config;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::warn;

// -- Public API -------------------------------------------------------------

/// Scope discriminator for config reads. `Community(id)` first, then
/// `Instance`, then the Rust const. v0 handlers pass `Instance` — no v0
/// code writes community-scoped rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
  Instance,
  Community(CommunityId),
}

/// Storage/wire type of a config value. Mirrors the `value_type` text column
/// on `governance_config`; the variants are the four primitive accessors plus
/// `Enum` for text-backed enumerations whose valid variants are pinned in
/// `ConfigKeyMetadata::valid_enum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
  Int,
  Float,
  Bool,
  Text,
  Enum,
}

/// Allowed scope layer for a key. `Both` means a write at `community:<id>`
/// overrides `instance`; `Instance` means the key is never community-scoped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigScope {
  Instance,
  Community,
  Both,
}

/// When a config change takes effect. `Immediate` is a cache invalidation;
/// `NextJuryCycle` means in-flight juries keep the old value via
/// `moderation_case.applied_config_snapshot`; `NextSnapshotJob` defers to the
/// next reputation snapshot tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyAt {
  Immediate,
  NextJuryCycle,
  NextSnapshotJob,
}

/// Static inclusive numeric range. Used for `valid_range` bounds on `Int` and
/// `Float` keys. Kept as two `f64` fields + `Copy` so the whole metadata
/// array stays `&'static` and compile-time-verifiable.
#[derive(Debug, Clone, Copy)]
pub struct NumericRange {
  pub min: f64,
  pub max: f64,
}

/// Compile-time metadata for one `governance_config` key. The array
/// `CONFIG_KEY_METADATA` holds one entry per seeded key; parity test
/// `every_seeded_key_has_metadata` rejects drift between
/// `SEEDED_KEYS_WITH_CONSTS` and this array. Editing a key's metadata is a
/// Rust-code change — there is no runtime override path, by NOT4 decision
/// 2026-04-19.
#[derive(Debug, Clone, Copy)]
pub struct ConfigKeyMetadata {
  pub key: &'static str,
  pub value_type: ValueType,
  pub valid_range: Option<NumericRange>,
  pub valid_enum: Option<&'static [&'static str]>,
  pub scope: ConfigScope,
  pub requires_re_jury: bool,
  pub requires_step_up: bool,
  pub apply_at_default: ApplyAt,
  pub description: &'static str,
  pub doc_anchor: &'static str,
}

impl Scope {
  /// Canonical wire representation — `"instance"` or `"community:<id>"`.
  /// Matches the `governance_config.scope` column literal and the shell
  /// wrapper's payload (`scripts/brehon/admin-config-write.sh:146-157`).
  /// `pub` since v1-AD-b's `admin_config` handler needs it for both
  /// policy dispatch logging and log-payload construction.
  pub fn as_str(self) -> Cow<'static, str> {
    match self {
      Scope::Instance => Cow::Borrowed("instance"),
      Scope::Community(CommunityId(id)) => Cow::Owned(format!("community:{id}")),
    }
  }

  /// Parse a wire-format scope string. Mirror of [`Scope::as_str`] —
  /// accepts exactly `"instance"` or `"community:<positive i32>"` with no
  /// whitespace. Returns a typed [`ScopeParseError`] on any other shape
  /// so the caller can emit a clean 400 and distinguish malformed input
  /// from a non-positive community id.
  pub fn parse_wire(s: &str) -> Result<Self, ScopeParseError> {
    if s == "instance" {
      return Ok(Scope::Instance);
    }
    let Some(rest) = s.strip_prefix("community:") else {
      return Err(ScopeParseError::Malformed(s.to_owned()));
    };
    let id: i32 = rest
      .parse()
      .map_err(|_parse_err| ScopeParseError::Malformed(s.to_owned()))?;
    if id < 1 {
      return Err(ScopeParseError::NonPositiveCommunityId(id));
    }
    Ok(Scope::Community(CommunityId(id)))
  }
}

/// Error returned by [`Scope::parse_wire`]. Keeps a narrow shape so tests can
/// assert on equality; malformed input currently carries the original wire text
/// internally (suppressed in `Display` per ADR-015 — see impl below).
/// See task 1 GOTCHA in the v1-AD-c plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeParseError {
  /// Input did not match either `"instance"` or `"community:<int>"`.
  Malformed(String),
  /// Input matched `"community:<int>"` but the int was <= 0.
  NonPositiveCommunityId(i32),
}

impl std::fmt::Display for ScopeParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      // Raw `s` is untrusted user input from the wire; do not echo it back
      // into the response. ADR-015: every user-visible string is scrubbed
      // before leaving the handler, and the simplest form of scrub for an
      // arbitrary-bytes field is to drop it entirely.
      ScopeParseError::Malformed(_) => write!(
        f,
        "scope is not recognised — expected `instance` or `community:<positive int>`"
      ),
      ScopeParseError::NonPositiveCommunityId(n) => {
        write!(f, "community_id must be >= 1; got {n}")
      }
    }
  }
}

/// One cached value. Matches the row shape — only one variant populated.
///
/// `Absent` records a completed fetch whose cascade found no row, distinguishing
/// "checked, nothing there" from "not yet queried" for the `get_*_opt` family.
/// Typed non-opt accessors keep the `if let Some(CachedValue::<T>(_))` early-return
/// pattern, so they naturally miss on an `Absent` entry and re-fetch — which
/// then re-caches `Absent` and proceeds to the `const_default_*` branch.
#[derive(Debug, Clone)]
enum CachedValue {
  Int(i64),
  Float(f64),
  Bool(bool),
  Text(String),
  Absent,
}

/// Per-request memo cache. Keys are `(scope_repr, key)` tuples.
#[derive(Debug, Default)]
pub struct ConfigCache {
  entries: HashMap<(String, String), CachedValue>,
}

impl ConfigCache {
  pub fn new() -> Self {
    Self::default()
  }
}

// -- Typed accessors --------------------------------------------------------

/// Read an `int` config value. Cascades `community:<id>` → `instance` → const.
///
/// Returns the `DEFAULT_*` const for this key if no row matches — i.e.
/// admin row deletion degrades gracefully, not into an error.
pub async fn get_int(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<i64> {
  let scope_repr = scope.as_str();
  if let Some(CachedValue::Int(v)) = cache
    .entries
    .get(&(scope_repr.as_ref().to_string(), key.to_string()))
  {
    return Ok(*v);
  }
  let v = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Int(v)) => v,
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as int but stored as {other:?}"
      ))
      .into());
    }
    None => const_default_int(key).ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` missing from DB and has no Rust const default"
      ))
    })?,
  };
  cache
    .entries
    .insert((scope_repr.into_owned(), key.to_string()), CachedValue::Int(v));
  Ok(v)
}

pub async fn get_float(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<f64> {
  let scope_repr = scope.as_str();
  if let Some(CachedValue::Float(v)) = cache
    .entries
    .get(&(scope_repr.as_ref().to_string(), key.to_string()))
  {
    return Ok(*v);
  }
  let v = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Float(v)) => v,
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as float but stored as {other:?}"
      ))
      .into());
    }
    None => const_default_float(key).ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` missing from DB and has no Rust const default"
      ))
    })?,
  };
  cache
    .entries
    .insert((scope_repr.into_owned(), key.to_string()), CachedValue::Float(v));
  Ok(v)
}

pub async fn get_bool(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<bool> {
  let scope_repr = scope.as_str();
  if let Some(CachedValue::Bool(v)) = cache
    .entries
    .get(&(scope_repr.as_ref().to_string(), key.to_string()))
  {
    return Ok(*v);
  }
  let v = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Bool(v)) => v,
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as bool but stored as {other:?}"
      ))
      .into());
    }
    None => const_default_bool(key).ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` missing from DB and has no Rust const default"
      ))
    })?,
  };
  cache
    .entries
    .insert((scope_repr.into_owned(), key.to_string()), CachedValue::Bool(v));
  Ok(v)
}

pub async fn get_text(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<String> {
  let scope_repr = scope.as_str();
  if let Some(CachedValue::Text(v)) = cache
    .entries
    .get(&(scope_repr.as_ref().to_string(), key.to_string()))
  {
    return Ok(v.clone());
  }
  let v = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Text(v)) => v,
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as text but stored as {other:?}"
      ))
      .into());
    }
    None => const_default_text(key).ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` missing from DB and has no Rust const default"
      ))
    })?,
  };
  cache
    .entries
    .insert((scope_repr.into_owned(), key.to_string()), CachedValue::Text(v.clone()));
  Ok(v)
}

// -- Cascade accessors (v1-JM-b) -------------------------------------------
//
// Dotted-namespace cascade: `<namespace>.<seg1>.<seg2>...` falls back to
// `<namespace>.<seg2>...` and so on down to the bare `<namespace>` key,
// finally to the Rust const default on the bare namespace. Each level runs
// through `fetch_value`, so the scope cascade (community → instance)
// applies at every level independently.
//
// Returns the value found at the MOST specific level that has a row (or a
// const default at the bare namespace). Absence at a more specific level
// does NOT fall through to a less-specific const — the const-default is
// only consulted after every DB level has been probed. Example cascade for
// `jury.panel_size` + `["founder", "severe"]`:
//
//   1. DB `jury.panel_size.founder.severe`
//   2. DB `jury.panel_size.severe`
//   3. DB `jury.panel_size`
//   4. const `DEFAULT_JURY_PANEL_SIZE` (via `const_default_int("jury.panel_size")`)
//
// Only added for Int and Float — the PRD §3.5 / §4 cascade pattern applies
// to numeric knobs (panel size, quorum fraction, threshold fraction) only.
// Adding `get_bool_cascade` / `get_text_cascade` is YAGNI per plan §10.2.

/// Walk a dotted-namespace cascade to resolve an int config key. See module
/// docs for the cascade shape. `namespace` is the bare key (e.g.
/// `"jury.panel_size"`); `segments` are the discriminating segments from
/// most-specific to least-specific (e.g. `&["founder", "severe"]`).
///
/// Each candidate is cached independently; a cache hit at any level short-
/// circuits the walk. Absent rows at a level are NOT cached (to keep the
/// hot-path simple — the seed migration covers the common case, and a
/// post-seed delete is rare enough that re-querying is acceptable).
pub async fn get_int_cascade(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  namespace: &str,
  segments: &[&str],
) -> LemmyResult<i64> {
  // Build the candidate list: most-specific first, then progressive tail
  // strips, then the bare namespace. `segments = &["founder", "severe"]`
  // yields `["<ns>.founder.severe", "<ns>.severe", "<ns>"]`.
  let mut candidates: Vec<String> = Vec::with_capacity(segments.len() + 1);
  if !segments.is_empty() {
    candidates.push(format!("{namespace}.{}", segments.join(".")));
    for i in 1..segments.len() {
      candidates.push(format!(
        "{namespace}.{}",
        segments.get(i..).unwrap_or(&[]).join(".")
      ));
    }
  }
  candidates.push(namespace.to_string());

  for candidate in &candidates {
    let scope_repr = scope.as_str();
    let cache_key = (scope_repr.as_ref().to_string(), candidate.clone());
    if let Some(CachedValue::Int(v)) = cache.entries.get(&cache_key) {
      return Ok(*v);
    }
    match fetch_value(pool, scope, candidate).await? {
      Some(CachedValue::Int(v)) => {
        cache
          .entries
          .insert((scope_repr.into_owned(), candidate.clone()), CachedValue::Int(v));
        return Ok(v);
      }
      Some(other) => {
        return Err(LemmyErrorType::Unknown(format!(
          "governance_config key `{candidate}` requested as int but stored as {other:?}"
        ))
        .into());
      }
      None => continue,
    }
  }

  candidates
    .iter()
    .find_map(|c| const_default_int(c))
    .ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "cascade walked {candidates:?} — no DB row found and no Rust const default for namespace `{namespace}`"
      ))
      .into()
    })
}

/// Float variant of [`get_int_cascade`]. Same contract: most-specific first,
/// bare-namespace last, const default on miss of every DB level.
pub async fn get_float_cascade(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  namespace: &str,
  segments: &[&str],
) -> LemmyResult<f64> {
  let mut candidates: Vec<String> = Vec::with_capacity(segments.len() + 1);
  if !segments.is_empty() {
    candidates.push(format!("{namespace}.{}", segments.join(".")));
    for i in 1..segments.len() {
      candidates.push(format!(
        "{namespace}.{}",
        segments.get(i..).unwrap_or(&[]).join(".")
      ));
    }
  }
  candidates.push(namespace.to_string());

  for candidate in &candidates {
    let scope_repr = scope.as_str();
    let cache_key = (scope_repr.as_ref().to_string(), candidate.clone());
    if let Some(CachedValue::Float(v)) = cache.entries.get(&cache_key) {
      return Ok(*v);
    }
    match fetch_value(pool, scope, candidate).await? {
      Some(CachedValue::Float(v)) => {
        cache
          .entries
          .insert((scope_repr.into_owned(), candidate.clone()), CachedValue::Float(v));
        return Ok(v);
      }
      Some(other) => {
        return Err(LemmyErrorType::Unknown(format!(
          "governance_config key `{candidate}` requested as float but stored as {other:?}"
        ))
        .into());
      }
      None => continue,
    }
  }

  candidates
    .iter()
    .find_map(|c| const_default_float(c))
    .ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "cascade walked {candidates:?} — no DB row found and no Rust const default for namespace `{namespace}`"
      ))
      .into()
    })
}

// -- Opt accessors (v1-AD-b) -----------------------------------------------
//
// These mirror the typed accessors above but return `Ok(None)` when the
// cascade finds no row, instead of falling through to a `const_default_*`.
// Required by v1-AD-c rule-set handlers that read keys with NO seed AND NO
// Rust const default — e.g. `rule_set.active_version_id`, where absence-of-
// row is the "no active version" signal (v1-AD-a advisor edit #2).
//
// Callers wanting a default on miss should call the non-opt `get_<type>`,
// which already does `None => const_default_<type>(key).ok_or_else(...)`.

/// Opt variant of `get_int`. Returns `Ok(None)` when no row matches; never
/// falls through to `const_default_int`. Caches the outcome as
/// `CachedValue::Absent` on `None` so repeat lookups are O(1).
pub async fn get_int_opt(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<Option<i64>> {
  let scope_repr = scope.as_str();
  let cache_key = (scope_repr.as_ref().to_string(), key.to_string());

  if let Some(entry) = cache.entries.get(&cache_key) {
    match entry {
      CachedValue::Int(v) => return Ok(Some(*v)),
      CachedValue::Absent => return Ok(None),
      other => {
        return Err(LemmyErrorType::Unknown(format!(
          "governance_config key `{key}` requested as int_opt but stored as {other:?}"
        ))
        .into());
      }
    }
  }

  let result = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Int(v)) => Some(v),
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as int_opt but stored as {other:?}"
      ))
      .into());
    }
    None => None,
  };

  let cached = match result {
    Some(v) => CachedValue::Int(v),
    None => CachedValue::Absent,
  };
  cache.entries.insert(cache_key, cached);
  Ok(result)
}

/// Opt variant of `get_float`. Same contract as `get_int_opt`.
pub async fn get_float_opt(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<Option<f64>> {
  let scope_repr = scope.as_str();
  let cache_key = (scope_repr.as_ref().to_string(), key.to_string());

  if let Some(entry) = cache.entries.get(&cache_key) {
    match entry {
      CachedValue::Float(v) => return Ok(Some(*v)),
      CachedValue::Absent => return Ok(None),
      other => {
        return Err(LemmyErrorType::Unknown(format!(
          "governance_config key `{key}` requested as float_opt but stored as {other:?}"
        ))
        .into());
      }
    }
  }

  let result = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Float(v)) => Some(v),
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as float_opt but stored as {other:?}"
      ))
      .into());
    }
    None => None,
  };

  let cached = match result {
    Some(v) => CachedValue::Float(v),
    None => CachedValue::Absent,
  };
  cache.entries.insert(cache_key, cached);
  Ok(result)
}

/// Opt variant of `get_bool`. Same contract as `get_int_opt`.
pub async fn get_bool_opt(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<Option<bool>> {
  let scope_repr = scope.as_str();
  let cache_key = (scope_repr.as_ref().to_string(), key.to_string());

  if let Some(entry) = cache.entries.get(&cache_key) {
    match entry {
      CachedValue::Bool(v) => return Ok(Some(*v)),
      CachedValue::Absent => return Ok(None),
      other => {
        return Err(LemmyErrorType::Unknown(format!(
          "governance_config key `{key}` requested as bool_opt but stored as {other:?}"
        ))
        .into());
      }
    }
  }

  let result = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Bool(v)) => Some(v),
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as bool_opt but stored as {other:?}"
      ))
      .into());
    }
    None => None,
  };

  let cached = match result {
    Some(v) => CachedValue::Bool(v),
    None => CachedValue::Absent,
  };
  cache.entries.insert(cache_key, cached);
  Ok(result)
}

/// Opt variant of `get_text`. Same contract as `get_int_opt`.
pub async fn get_text_opt(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<Option<String>> {
  let scope_repr = scope.as_str();
  let cache_key = (scope_repr.as_ref().to_string(), key.to_string());

  if let Some(entry) = cache.entries.get(&cache_key) {
    match entry {
      CachedValue::Text(v) => return Ok(Some(v.clone())),
      CachedValue::Absent => return Ok(None),
      other => {
        return Err(LemmyErrorType::Unknown(format!(
          "governance_config key `{key}` requested as text_opt but stored as {other:?}"
        ))
        .into());
      }
    }
  }

  let result = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Text(v)) => Some(v),
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as text_opt but stored as {other:?}"
      ))
      .into());
    }
    None => None,
  };

  let cached = match &result {
    Some(v) => CachedValue::Text(v.clone()),
    None => CachedValue::Absent,
  };
  cache.entries.insert(cache_key, cached);
  Ok(result)
}

// -- Membership state parser (task 51) --------------------------------------

/// Parse the `onboarding.default_membership_state` config value into a
/// `MembershipState` enum. Exhaustive match on the three v0 variants;
/// unknown text warns and falls back to `Member`. Placed here, not in task
/// 51's register handler, so config parsers stay colocated with the config
/// reader itself (GOTCHA-51c).
pub fn parse_membership_state(s: &str) -> MembershipState {
  match s {
    "member" => MembershipState::Member,
    "provisional" => MembershipState::Provisional,
    "suspended" => MembershipState::Suspended,
    other => {
      warn!("unknown membership_state config value '{other}' — falling back to 'member'");
      MembershipState::Member
    }
  }
}

// -- Private helpers --------------------------------------------------------

/// Tuple shape for the `governance_config` row load. Module-scope
/// because `items-after-statements` is denied at workspace level.
type ConfigRow = (
  String,
  Option<i64>,
  Option<f64>,
  Option<bool>,
  Option<String>,
);

/// Fetch a single value from `governance_config` — the view already
/// returns the most-recent row per `(scope, key)`. Returns `None` if no row
/// matches the requested scope.
///
/// Fallback cascade: if `scope = Community(id)` returns None, retry with
/// `Instance` before returning None to the caller.
async fn fetch_value(
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<Option<CachedValue>> {
  if let Scope::Community(_) = scope {
    if let Some(v) = fetch_value_at_scope(pool, scope.as_str(), key).await? {
      return Ok(Some(v));
    }
    return fetch_value_at_scope(pool, Scope::Instance.as_str(), key).await;
  }
  fetch_value_at_scope(pool, scope.as_str(), key).await
}

async fn fetch_value_at_scope(
  pool: &mut DbPool<'_>,
  scope_str: Cow<'static, str>,
  key: &str,
) -> LemmyResult<Option<CachedValue>> {
  let conn = &mut get_conn(pool).await?;

  // governance_config is append-only with multiple rows per (scope, key)
  // keyed by valid_from. ORDER BY valid_from DESC + LIMIT 1 (.first) is
  // load-bearing — without it Postgres returns arbitrary order and reads
  // can return stale seeded rows instead of admin_set_config writes.
  // Regression history: commit 8e3bba1 dropped the governance_config_current
  // view; this code path needs to do the latest-wins ordering itself.
  let row: Option<ConfigRow> = governance_config::table
    .filter(governance_config::scope.eq(scope_str.into_owned()))
    .filter(governance_config::key.eq(key))
    .select((
      governance_config::value_type,
      governance_config::value_int,
      governance_config::value_float,
      governance_config::value_bool,
      governance_config::value_text,
    ))
    .order_by(governance_config::valid_from.desc())
    .first::<ConfigRow>(conn)
    .await
    .optional()?;

  Ok(row.and_then(|(vtype, vi, vf, vb, vt)| match vtype.as_str() {
    "int" => vi.map(CachedValue::Int),
    "float" => vf.map(CachedValue::Float),
    "bool" => vb.map(CachedValue::Bool),
    "text" => vt.map(CachedValue::Text),
    _ => None,
  }))
}

// -- Const defaults ---------------------------------------------------------
//
// Every `pub const DEFAULT_*` below mirrors a seeded row in the task 50
// migration. The `parity::SEEDED_KEYS_WITH_CONSTS` list + structural test
// enforce the one-to-one mapping.

pub const DEFAULT_THRESHOLDS_JURY_RELIABILITY: i64 = 50;
pub const DEFAULT_THRESHOLDS_REPORTING_ACCURACY: i64 = 50;
pub const DEFAULT_THRESHOLDS_ENDORSEMENT_STRENGTH: i64 = 25;
pub const DEFAULT_JURY_PANEL_SIZE: i64 = 5;
pub const DEFAULT_JURY_QUORUM: i64 = 3;
pub const DEFAULT_JURY_AGE_REQUIREMENT_DAYS: i64 = 60;
/// Per-case cap on how many jurors on a single panel may be assigned concurrently.
/// Pairs with `DEFAULT_JURY_MAX_CONCURRENT_ASSIGNMENTS_PER_JUROR_TOTAL` (authoritative
/// for global cross-case juror load); this constant bounds within-panel concurrency only.
pub const DEFAULT_JURY_MAX_CONCURRENT_ASSIGNMENTS: i64 = 3;
pub const DEFAULT_JURY_FALLBACK_ON_SMALL_POOL: bool = true;
pub const DEFAULT_DELTAS_JUROR_ALIGNED: i64 = 10;
pub const DEFAULT_DELTAS_JUROR_OUTLIER: i64 = -5;
pub const DEFAULT_DELTAS_REPORTER_UPHELD: i64 = 10;
pub const DEFAULT_DELTAS_REPORTER_DISMISSED: i64 = -5;
pub const DEFAULT_DELTAS_ENDORSEMENT_CREATED_SPONSOR: i64 = 5;
pub const DEFAULT_DELTAS_ENDORSEMENT_CREATED_SPONSEE: i64 = 5;
pub const DEFAULT_DELTAS_SPONSOR_LIABILITY_MINOR: i64 = -10;
pub const DEFAULT_DELTAS_SPONSOR_LIABILITY_MODERATE: i64 = -50;
pub const DEFAULT_DELTAS_SPONSOR_LIABILITY_SEVERE: i64 = -200;
pub const DEFAULT_LIABILITY_FOUNDER_MULTIPLIER: f64 = 2.0;
pub const DEFAULT_LIABILITY_REGULAR_MULTIPLIER: f64 = 1.0;
pub const DEFAULT_LIABILITY_SPONSOR_LIABILITY_FLOOR: i64 = 0;
pub const DEFAULT_REPORT_BASE_WEIGHT: f64 = 1.0;
pub const DEFAULT_REPORT_CLAMP_MIN: f64 = 0.1;
pub const DEFAULT_REPORT_CLAMP_MAX: f64 = 2.0;
pub const DEFAULT_REPORT_RECENCY_HALF_LIFE_HOURS: f64 = 168.0;
pub const DEFAULT_REPORT_CASE_THRESHOLD_MICROS: i64 = 3_000_000;
pub const DEFAULT_DECAY_POSITIVE_HALF_LIFE_DAYS: i64 = 90;
pub const DEFAULT_ONBOARDING_DEFAULT_MEMBERSHIP_STATE: &str = "member";
pub const DEFAULT_ONBOARDING_SPONSOR_GATE_STRATEGY: &str = "age";
pub const DEFAULT_ONBOARDING_SPONSOR_MIN_ACCOUNT_AGE_DAYS: i64 = 30;
pub const DEFAULT_FOUNDER_MAX_FOUNDERS_ACTIVE: i64 = 20;
pub const DEFAULT_FOUNDER_MAX_EXPIRES_DAYS: i64 = 365;
pub const DEFAULT_FOUNDER_MAX_SEED_DELTA: i64 = 200;
pub const DEFAULT_JOB_SNAPSHOT_INTERVAL_SECONDS: i64 = 900;
pub const DEFAULT_JOB_SNAPSHOT_BATCH_CHUNK_SIZE: i64 = 500;

// -- v1-AD-a additions (admin dashboard foundation sub-phase) ---------------
//
// 27 new keys seeded by migration `2026-04-22-000300-0000_seed_v1_config_keys`.
// Authoritative list = PRD §5.2 minus 10 sponsor-liability-owned rows (land in
// sponsor-liability-v1) minus 1 `rule_set.active_version_id` (deliberately
// un-seeded per plan §4.1 — absence-of-row IS the "no active version" signal).
// 2 governance.dashboard.* keys are included per PRD §5.2; the plan §13 task 6
// sample list omitted them — advisor edit #1 reconciliation point.

pub const DEFAULT_JURY_SEVERITY_THRESHOLDS_MINOR: &str = "majority";
pub const DEFAULT_JURY_SEVERITY_THRESHOLDS_MODERATE: &str = "60%";
pub const DEFAULT_JURY_SEVERITY_THRESHOLDS_SEVERE: &str = "75%";
pub const DEFAULT_JURY_DIVERSITY_CONSTRAINTS_ENABLED: bool = true;
pub const DEFAULT_JURY_APPEAL_PANEL_SIZE_INCREASE: i64 = 2;
pub const DEFAULT_JURY_DEADLINE_WINDOW_HOURS: i64 = 72;
pub const DEFAULT_DECAY_NEGATIVE_HALF_LIFE_DAYS: i64 = 180;
pub const DEFAULT_DECAY_ENDORSEMENT_STRENGTH_HALF_LIFE_DAYS: i64 = 90;
pub const DEFAULT_DECAY_JURY_RELIABILITY_HALF_LIFE_DAYS: i64 = 90;
pub const DEFAULT_ONBOARDING_SPONSOR_MIN_ENDORSEMENT_STRENGTH: i64 = 25;
pub const DEFAULT_ONBOARDING_SPONSOR_ALLOWLIST_TABLE_NAME: &str = "sponsor_allowlist";
pub const DEFAULT_ONBOARDING_PROVISIONAL_MEMBERSHIP_COOLDOWN_DAYS: i64 = 14;
pub const DEFAULT_FOUNDER_FOUNDER_SEAL_VISIBLE_IN_PROFILE: bool = true;
pub const DEFAULT_DELTAS_PARTICIPATION_WEEKLY_ACTIVE: i64 = 1;
pub const DEFAULT_PARTICIPATION_DORMANCY_WINDOW_DAYS: i64 = 30;
pub const DEFAULT_DELTAS_PARTICIPATION_DORMANT: i64 = -2;
pub const DEFAULT_PARTICIPATION_ATTESTATION_ENABLED: bool = false;
pub const DEFAULT_FEDERATION_INBOUND_ADVISORY_ONLY: bool = true;
pub const DEFAULT_FEDERATION_PEER_ATTESTATION_TTL_DAYS: i64 = 30;
pub const DEFAULT_FEDERATION_SIGNATURE_REQUIRED: bool = true;
pub const DEFAULT_FEDERATION_QUARANTINE_RECOMMENDATION_SEVERITY_FLOOR: &str = "moderate";
pub const DEFAULT_FEDERATION_OUTBOUND_PUBLISH_ENABLED: bool = true;
pub const DEFAULT_RULE_SET_AUTO_CARRY_IN_FLIGHT_CASES: bool = true;
pub const DEFAULT_RULE_SET_TEXT_MAX_BYTES: i64 = 65_536;
pub const DEFAULT_RULE_SET_VERSION_PROPAGATION_DELAY_HOURS: i64 = 24;
pub const DEFAULT_GOVERNANCE_DASHBOARD_HTML_PAGES_ENABLED: bool = true;
pub const DEFAULT_GOVERNANCE_DASHBOARD_STEP_UP_ENFORCED: bool = false;

// -- v1-JM-a additions (jury-mechanics sub-phase A) -------------------------
//
// 27 new keys seeded by migration `2026-04-23-000200-0000_seed_v1_jm_config_keys`.
// Authoritative list = PRD §10 defaults matrix. Breakdown: 9 panel_size cells
// (status × severity) + 3 quorum_fraction (per severity) + 3 threshold_fraction
// (per severity) + 5 jury.constraints.* (4 bool + 1 int cooldown) + 2 more
// jury.* (max_retries_before_relax int, max_concurrent_assignments_per_juror_total
// int) + 5 appeal.* (1 float multiplier + 3 int + 1 bool). Distinct from
// v1-AD-a's `jury.severity_thresholds.*` (text display strings) and
// `jury.diversity_constraints_enabled` (coarse toggle) — both coexist.

// jury.panel_size.<status>.<severity> — 9 keys (int)
pub const DEFAULT_JURY_PANEL_SIZE_REGULAR_MINOR: i64 = 5;
pub const DEFAULT_JURY_PANEL_SIZE_REGULAR_MODERATE: i64 = 5;
pub const DEFAULT_JURY_PANEL_SIZE_REGULAR_SEVERE: i64 = 7;
pub const DEFAULT_JURY_PANEL_SIZE_FOUNDER_MINOR: i64 = 5;
pub const DEFAULT_JURY_PANEL_SIZE_FOUNDER_MODERATE: i64 = 7;
pub const DEFAULT_JURY_PANEL_SIZE_FOUNDER_SEVERE: i64 = 9;
pub const DEFAULT_JURY_PANEL_SIZE_PROBATION_MINOR: i64 = 3;
pub const DEFAULT_JURY_PANEL_SIZE_PROBATION_MODERATE: i64 = 5;
pub const DEFAULT_JURY_PANEL_SIZE_PROBATION_SEVERE: i64 = 5;

// jury.quorum_fraction.<severity> — 3 keys (float)
pub const DEFAULT_JURY_QUORUM_FRACTION_MINOR: f64 = 0.6;
pub const DEFAULT_JURY_QUORUM_FRACTION_MODERATE: f64 = 0.6;
pub const DEFAULT_JURY_QUORUM_FRACTION_SEVERE: f64 = 0.71;

// jury.threshold_fraction.<severity> — 3 keys (float)
pub const DEFAULT_JURY_THRESHOLD_FRACTION_MINOR: f64 = 0.5001;
pub const DEFAULT_JURY_THRESHOLD_FRACTION_MODERATE: f64 = 0.6;
pub const DEFAULT_JURY_THRESHOLD_FRACTION_SEVERE: f64 = 0.75;

// jury.constraints.* — 5 keys (4 bool + 1 int)
pub const DEFAULT_JURY_CONSTRAINTS_NO_MAJORITY_FROM_SAME_SPONSOR_CLUSTER: bool = true;
pub const DEFAULT_JURY_CONSTRAINTS_GEOGRAPHIC_DIVERSITY_PREFERRED: bool = true;
pub const DEFAULT_JURY_CONSTRAINTS_NO_RECENT_JUROR_REPEAT: bool = true;
pub const DEFAULT_JURY_CONSTRAINTS_JUROR_COOLDOWN_DAYS: i64 = 7;
pub const DEFAULT_JURY_CONSTRAINTS_NO_SAME_ENDORSEMENT_CHAIN: bool = false;

// jury.constraints.max_retries_before_relax + jury.max_concurrent_* — 2 keys (int)
pub const DEFAULT_JURY_CONSTRAINTS_MAX_RETRIES_BEFORE_RELAX: i64 = 5;
/// Global cross-case cap on concurrent assignments per juror. Authoritative for
/// overall juror-load throttling across the whole network; `DEFAULT_JURY_MAX_CONCURRENT_ASSIGNMENTS`
/// (above) bounds within-panel concurrency for a single case only.
pub const DEFAULT_JURY_MAX_CONCURRENT_ASSIGNMENTS_PER_JUROR_TOTAL: i64 = 2;

// appeal.* — 5 keys (1 float + 3 int + 1 bool)
pub const DEFAULT_APPEAL_PANEL_SIZE_MULTIPLIER: f64 = 1.5;
pub const DEFAULT_APPEAL_PANEL_SIZE_FLOOR_INCREMENT: i64 = 2;
pub const DEFAULT_APPEAL_THRESHOLD_TIER_BUMP: i64 = 1;
pub const DEFAULT_APPEAL_WINDOW_DAYS: i64 = 7;
pub const DEFAULT_APPEAL_AUTO_SELECT_ON_APPEAL_ACCEPTANCE: bool = true;

// -- v1-SL-a additions (sponsor-liability sub-phase A) ---------------
//
// 13 new keys seeded by migration `2026-05-03-000000-0000_add_sponsor_liability_grace_window`.
// Authoritative list = PRD §10 defaults matrix. Breakdown:
// 6 grace_window_*_hours (int) + 2 restoration escape (1 bool + 1 int)
// + 1 multi_sponsor_escape_rule (text) + 1 revoke_rate_limit_per_day (int)
// + 3 job.grace_check_* (2 int + 1 float) = 13 keys total.
//
// Per DQ #115 (advisor 2026-05-03): all hour keys are raw integer hours
// (NOT micros-scaled); used as Postgres `INTERVAL '<N> hours'` operands
// in PRD §6.2 scheduler + PRD §8.4 backfill SQL. The
// `feedback_brehon_config_micros_scaled.md` lesson scopes to
// reputation/score-formula math; wall-clock units are out of scope.
//
// Distinct from existing v0 `liability.*` keys (founder_multiplier,
// regular_multiplier, sponsor_liability_floor — lines 795-797). Both
// coexist; SL-a's keys all carry `liability.grace_window_*` /
// `liability.restoration_*` / `liability.multi_sponsor_*` /
// `liability.revoke_*` namespace prefixes per PRD §18 B4 key-rename
// table.

// liability.grace_window_*_hours — 6 keys (int; raw hours per DQ #115)
pub const DEFAULT_LIABILITY_GRACE_WINDOW_MINOR_HOURS: i64 = 24;
pub const DEFAULT_LIABILITY_GRACE_WINDOW_MODERATE_HOURS: i64 = 72;
pub const DEFAULT_LIABILITY_GRACE_WINDOW_SEVERE_HOURS: i64 = 168;
pub const DEFAULT_LIABILITY_GRACE_WINDOW_MINIMUM_HOURS: i64 = 1;
pub const DEFAULT_LIABILITY_GRACE_WINDOW_MAXIMUM_HOURS: i64 = 720;
pub const DEFAULT_LIABILITY_GRACE_WINDOW_ALERT_THRESHOLD_HOURS: i64 = 24;

// liability.restoration_* — 2 keys (1 bool + 1 int)
pub const DEFAULT_LIABILITY_RESTORATION_ESCAPES_LIABILITY: bool = true;
pub const DEFAULT_LIABILITY_RESTORATION_SEVERITY_REDUCTION_STEPS: i64 = 0;

// liability.multi_sponsor_escape_rule — 1 key (text; enum: any_revocation | all_revocation | majority_revocation)
pub const DEFAULT_LIABILITY_MULTI_SPONSOR_ESCAPE_RULE: &str = "any_revocation";

// liability.revoke_rate_limit_per_day — 1 key (int; per-user per-rolling-24h cap)
pub const DEFAULT_LIABILITY_REVOKE_RATE_LIMIT_PER_DAY: i64 = 5;

// job.grace_check_* — 3 keys (2 int + 1 float)
pub const DEFAULT_JOB_GRACE_CHECK_INTERVAL_MINUTES: i64 = 5;
pub const DEFAULT_JOB_GRACE_CHECK_BATCH_SIZE: i64 = 100;
pub const DEFAULT_JOB_GRACE_CHECK_STALENESS_ALERT_MULTIPLIER: f64 = 2.0;

pub(crate) fn const_default_int(key: &str) -> Option<i64> {
  match key {
    "thresholds.jury_reliability" => Some(DEFAULT_THRESHOLDS_JURY_RELIABILITY),
    "thresholds.reporting_accuracy" => Some(DEFAULT_THRESHOLDS_REPORTING_ACCURACY),
    "thresholds.endorsement_strength" => Some(DEFAULT_THRESHOLDS_ENDORSEMENT_STRENGTH),
    "jury.panel_size" => Some(DEFAULT_JURY_PANEL_SIZE),
    "jury.quorum" => Some(DEFAULT_JURY_QUORUM),
    "jury.age_requirement_days" => Some(DEFAULT_JURY_AGE_REQUIREMENT_DAYS),
    "jury.max_concurrent_assignments" => Some(DEFAULT_JURY_MAX_CONCURRENT_ASSIGNMENTS),
    "deltas.juror_aligned" => Some(DEFAULT_DELTAS_JUROR_ALIGNED),
    "deltas.juror_outlier" => Some(DEFAULT_DELTAS_JUROR_OUTLIER),
    "deltas.reporter_upheld" => Some(DEFAULT_DELTAS_REPORTER_UPHELD),
    "deltas.reporter_dismissed" => Some(DEFAULT_DELTAS_REPORTER_DISMISSED),
    "deltas.endorsement_created_sponsor" => Some(DEFAULT_DELTAS_ENDORSEMENT_CREATED_SPONSOR),
    "deltas.endorsement_created_sponsee" => Some(DEFAULT_DELTAS_ENDORSEMENT_CREATED_SPONSEE),
    "deltas.sponsor_liability_minor" => Some(DEFAULT_DELTAS_SPONSOR_LIABILITY_MINOR),
    "deltas.sponsor_liability_moderate" => Some(DEFAULT_DELTAS_SPONSOR_LIABILITY_MODERATE),
    "deltas.sponsor_liability_severe" => Some(DEFAULT_DELTAS_SPONSOR_LIABILITY_SEVERE),
    "liability.sponsor_liability_floor" => Some(DEFAULT_LIABILITY_SPONSOR_LIABILITY_FLOOR),
    "report.case_threshold_micros" => Some(DEFAULT_REPORT_CASE_THRESHOLD_MICROS),
    "decay.positive_half_life_days" => Some(DEFAULT_DECAY_POSITIVE_HALF_LIFE_DAYS),
    "onboarding.sponsor_min_account_age_days" => Some(DEFAULT_ONBOARDING_SPONSOR_MIN_ACCOUNT_AGE_DAYS),
    "founder.max_founders_active" => Some(DEFAULT_FOUNDER_MAX_FOUNDERS_ACTIVE),
    "founder.max_expires_days" => Some(DEFAULT_FOUNDER_MAX_EXPIRES_DAYS),
    "founder.max_seed_delta" => Some(DEFAULT_FOUNDER_MAX_SEED_DELTA),
    "job.snapshot_interval_seconds" => Some(DEFAULT_JOB_SNAPSHOT_INTERVAL_SECONDS),
    "job.snapshot_batch_chunk_size" => Some(DEFAULT_JOB_SNAPSHOT_BATCH_CHUNK_SIZE),
    // v1-AD-a additions
    "jury.appeal_panel_size_increase" => Some(DEFAULT_JURY_APPEAL_PANEL_SIZE_INCREASE),
    "jury.deadline_window_hours" => Some(DEFAULT_JURY_DEADLINE_WINDOW_HOURS),
    "decay.negative_half_life_days" => Some(DEFAULT_DECAY_NEGATIVE_HALF_LIFE_DAYS),
    "decay.endorsement_strength_half_life_days" => {
      Some(DEFAULT_DECAY_ENDORSEMENT_STRENGTH_HALF_LIFE_DAYS)
    }
    "decay.jury_reliability_half_life_days" => Some(DEFAULT_DECAY_JURY_RELIABILITY_HALF_LIFE_DAYS),
    "onboarding.sponsor_min_endorsement_strength" => {
      Some(DEFAULT_ONBOARDING_SPONSOR_MIN_ENDORSEMENT_STRENGTH)
    }
    "onboarding.provisional_membership_cooldown_days" => {
      Some(DEFAULT_ONBOARDING_PROVISIONAL_MEMBERSHIP_COOLDOWN_DAYS)
    }
    "deltas.participation_weekly_active" => Some(DEFAULT_DELTAS_PARTICIPATION_WEEKLY_ACTIVE),
    "participation.dormancy_window_days" => Some(DEFAULT_PARTICIPATION_DORMANCY_WINDOW_DAYS),
    "deltas.participation_dormant" => Some(DEFAULT_DELTAS_PARTICIPATION_DORMANT),
    "federation.peer_attestation_ttl_days" => Some(DEFAULT_FEDERATION_PEER_ATTESTATION_TTL_DAYS),
    "rule_set.text_max_bytes" => Some(DEFAULT_RULE_SET_TEXT_MAX_BYTES),
    "rule_set.version_propagation_delay_hours" => {
      Some(DEFAULT_RULE_SET_VERSION_PROPAGATION_DELAY_HOURS)
    }
    // v1-JM-a additions
    "jury.panel_size.regular.minor" => Some(DEFAULT_JURY_PANEL_SIZE_REGULAR_MINOR),
    "jury.panel_size.regular.moderate" => Some(DEFAULT_JURY_PANEL_SIZE_REGULAR_MODERATE),
    "jury.panel_size.regular.severe" => Some(DEFAULT_JURY_PANEL_SIZE_REGULAR_SEVERE),
    "jury.panel_size.founder.minor" => Some(DEFAULT_JURY_PANEL_SIZE_FOUNDER_MINOR),
    "jury.panel_size.founder.moderate" => Some(DEFAULT_JURY_PANEL_SIZE_FOUNDER_MODERATE),
    "jury.panel_size.founder.severe" => Some(DEFAULT_JURY_PANEL_SIZE_FOUNDER_SEVERE),
    "jury.panel_size.probation.minor" => Some(DEFAULT_JURY_PANEL_SIZE_PROBATION_MINOR),
    "jury.panel_size.probation.moderate" => Some(DEFAULT_JURY_PANEL_SIZE_PROBATION_MODERATE),
    "jury.panel_size.probation.severe" => Some(DEFAULT_JURY_PANEL_SIZE_PROBATION_SEVERE),
    "jury.constraints.juror_cooldown_days" => {
      Some(DEFAULT_JURY_CONSTRAINTS_JUROR_COOLDOWN_DAYS)
    }
    "jury.constraints.max_retries_before_relax" => {
      Some(DEFAULT_JURY_CONSTRAINTS_MAX_RETRIES_BEFORE_RELAX)
    }
    "jury.max_concurrent_assignments_per_juror_total" => {
      Some(DEFAULT_JURY_MAX_CONCURRENT_ASSIGNMENTS_PER_JUROR_TOTAL)
    }
    "appeal.panel_size_floor_increment" => Some(DEFAULT_APPEAL_PANEL_SIZE_FLOOR_INCREMENT),
    "appeal.threshold_tier_bump" => Some(DEFAULT_APPEAL_THRESHOLD_TIER_BUMP),
    "appeal.window_days" => Some(DEFAULT_APPEAL_WINDOW_DAYS),
    // v1-SL-a additions
    "liability.grace_window_minor_hours" => Some(DEFAULT_LIABILITY_GRACE_WINDOW_MINOR_HOURS),
    "liability.grace_window_moderate_hours" => Some(DEFAULT_LIABILITY_GRACE_WINDOW_MODERATE_HOURS),
    "liability.grace_window_severe_hours" => Some(DEFAULT_LIABILITY_GRACE_WINDOW_SEVERE_HOURS),
    "liability.grace_window_minimum_hours" => Some(DEFAULT_LIABILITY_GRACE_WINDOW_MINIMUM_HOURS),
    "liability.grace_window_maximum_hours" => Some(DEFAULT_LIABILITY_GRACE_WINDOW_MAXIMUM_HOURS),
    "liability.grace_window_alert_threshold_hours" => {
      Some(DEFAULT_LIABILITY_GRACE_WINDOW_ALERT_THRESHOLD_HOURS)
    }
    "liability.restoration_severity_reduction_steps" => {
      Some(DEFAULT_LIABILITY_RESTORATION_SEVERITY_REDUCTION_STEPS)
    }
    "liability.revoke_rate_limit_per_day" => Some(DEFAULT_LIABILITY_REVOKE_RATE_LIMIT_PER_DAY),
    "job.grace_check_interval_minutes" => Some(DEFAULT_JOB_GRACE_CHECK_INTERVAL_MINUTES),
    "job.grace_check_batch_size" => Some(DEFAULT_JOB_GRACE_CHECK_BATCH_SIZE),
    _ => None,
  }
}

pub(crate) fn const_default_float(key: &str) -> Option<f64> {
  match key {
    "liability.founder_multiplier" => Some(DEFAULT_LIABILITY_FOUNDER_MULTIPLIER),
    "liability.regular_multiplier" => Some(DEFAULT_LIABILITY_REGULAR_MULTIPLIER),
    "report.base_weight" => Some(DEFAULT_REPORT_BASE_WEIGHT),
    "report.clamp_min" => Some(DEFAULT_REPORT_CLAMP_MIN),
    "report.clamp_max" => Some(DEFAULT_REPORT_CLAMP_MAX),
    "report.recency_half_life_hours" => Some(DEFAULT_REPORT_RECENCY_HALF_LIFE_HOURS),
    // v1-JM-a additions
    "jury.quorum_fraction.minor" => Some(DEFAULT_JURY_QUORUM_FRACTION_MINOR),
    "jury.quorum_fraction.moderate" => Some(DEFAULT_JURY_QUORUM_FRACTION_MODERATE),
    "jury.quorum_fraction.severe" => Some(DEFAULT_JURY_QUORUM_FRACTION_SEVERE),
    "jury.threshold_fraction.minor" => Some(DEFAULT_JURY_THRESHOLD_FRACTION_MINOR),
    "jury.threshold_fraction.moderate" => Some(DEFAULT_JURY_THRESHOLD_FRACTION_MODERATE),
    "jury.threshold_fraction.severe" => Some(DEFAULT_JURY_THRESHOLD_FRACTION_SEVERE),
    "appeal.panel_size_multiplier" => Some(DEFAULT_APPEAL_PANEL_SIZE_MULTIPLIER),
    // v1-SL-a additions
    "job.grace_check_staleness_alert_multiplier" => {
      Some(DEFAULT_JOB_GRACE_CHECK_STALENESS_ALERT_MULTIPLIER)
    }
    _ => None,
  }
}

pub(crate) fn const_default_bool(key: &str) -> Option<bool> {
  match key {
    "jury.fallback_on_small_pool" => Some(DEFAULT_JURY_FALLBACK_ON_SMALL_POOL),
    // v1-AD-a additions
    "jury.diversity_constraints_enabled" => Some(DEFAULT_JURY_DIVERSITY_CONSTRAINTS_ENABLED),
    "founder.founder_seal_visible_in_profile" => {
      Some(DEFAULT_FOUNDER_FOUNDER_SEAL_VISIBLE_IN_PROFILE)
    }
    "participation.attestation_enabled" => Some(DEFAULT_PARTICIPATION_ATTESTATION_ENABLED),
    "federation.inbound_advisory_only" => Some(DEFAULT_FEDERATION_INBOUND_ADVISORY_ONLY),
    "federation.signature_required" => Some(DEFAULT_FEDERATION_SIGNATURE_REQUIRED),
    "federation.outbound_publish_enabled" => Some(DEFAULT_FEDERATION_OUTBOUND_PUBLISH_ENABLED),
    "rule_set.auto_carry_in_flight_cases" => Some(DEFAULT_RULE_SET_AUTO_CARRY_IN_FLIGHT_CASES),
    "governance.dashboard.html_pages_enabled" => {
      Some(DEFAULT_GOVERNANCE_DASHBOARD_HTML_PAGES_ENABLED)
    }
    "governance.dashboard.step_up_enforced" => {
      Some(DEFAULT_GOVERNANCE_DASHBOARD_STEP_UP_ENFORCED)
    }
    // v1-JM-a additions
    "jury.constraints.no_majority_from_same_sponsor_cluster" => {
      Some(DEFAULT_JURY_CONSTRAINTS_NO_MAJORITY_FROM_SAME_SPONSOR_CLUSTER)
    }
    "jury.constraints.geographic_diversity_preferred" => {
      Some(DEFAULT_JURY_CONSTRAINTS_GEOGRAPHIC_DIVERSITY_PREFERRED)
    }
    "jury.constraints.no_recent_juror_repeat" => {
      Some(DEFAULT_JURY_CONSTRAINTS_NO_RECENT_JUROR_REPEAT)
    }
    "jury.constraints.no_same_endorsement_chain" => {
      Some(DEFAULT_JURY_CONSTRAINTS_NO_SAME_ENDORSEMENT_CHAIN)
    }
    "appeal.auto_select_on_appeal_acceptance" => {
      Some(DEFAULT_APPEAL_AUTO_SELECT_ON_APPEAL_ACCEPTANCE)
    }
    // v1-SL-a additions
    "liability.restoration_escapes_liability" => {
      Some(DEFAULT_LIABILITY_RESTORATION_ESCAPES_LIABILITY)
    }
    _ => None,
  }
}

pub(crate) fn const_default_text(key: &str) -> Option<String> {
  match key {
    "onboarding.default_membership_state" => {
      Some(DEFAULT_ONBOARDING_DEFAULT_MEMBERSHIP_STATE.to_string())
    }
    "onboarding.sponsor_gate_strategy" => {
      Some(DEFAULT_ONBOARDING_SPONSOR_GATE_STRATEGY.to_string())
    }
    // v1-AD-a additions
    "jury.severity_thresholds.minor" => {
      Some(DEFAULT_JURY_SEVERITY_THRESHOLDS_MINOR.to_string())
    }
    "jury.severity_thresholds.moderate" => {
      Some(DEFAULT_JURY_SEVERITY_THRESHOLDS_MODERATE.to_string())
    }
    "jury.severity_thresholds.severe" => {
      Some(DEFAULT_JURY_SEVERITY_THRESHOLDS_SEVERE.to_string())
    }
    "onboarding.sponsor_allowlist_table_name" => {
      Some(DEFAULT_ONBOARDING_SPONSOR_ALLOWLIST_TABLE_NAME.to_string())
    }
    "federation.quarantine_recommendation_severity_floor" => {
      Some(DEFAULT_FEDERATION_QUARANTINE_RECOMMENDATION_SEVERITY_FLOOR.to_string())
    }
    // v1-SL-a additions
    "liability.multi_sponsor_escape_rule" => {
      Some(DEFAULT_LIABILITY_MULTI_SPONSOR_ESCAPE_RULE.to_string())
    }
    _ => None,
  }
}

// -- Parity list (Watch 1) --------------------------------------------------

/// The canonical seeded-keys list. Each tuple is `(key, const_name, value_type)`.
/// The `const_name` is not consumed at runtime — it exists for the structural
/// parity test and for grep-ability (decision-queue #14 carry-over rationale
/// in GOTCHA-50i). `pub` so `crates/server/tests/e2e.rs::config_parity_round_trip`
/// can walk it without re-declaring.
pub const SEEDED_KEYS_WITH_CONSTS: &[(&str, &str, &str)] = &[
  ("thresholds.jury_reliability", "DEFAULT_THRESHOLDS_JURY_RELIABILITY", "int"),
  ("thresholds.reporting_accuracy", "DEFAULT_THRESHOLDS_REPORTING_ACCURACY", "int"),
  ("thresholds.endorsement_strength", "DEFAULT_THRESHOLDS_ENDORSEMENT_STRENGTH", "int"),
  ("jury.panel_size", "DEFAULT_JURY_PANEL_SIZE", "int"),
  ("jury.quorum", "DEFAULT_JURY_QUORUM", "int"),
  ("jury.age_requirement_days", "DEFAULT_JURY_AGE_REQUIREMENT_DAYS", "int"),
  ("jury.max_concurrent_assignments", "DEFAULT_JURY_MAX_CONCURRENT_ASSIGNMENTS", "int"),
  ("jury.fallback_on_small_pool", "DEFAULT_JURY_FALLBACK_ON_SMALL_POOL", "bool"),
  ("deltas.juror_aligned", "DEFAULT_DELTAS_JUROR_ALIGNED", "int"),
  ("deltas.juror_outlier", "DEFAULT_DELTAS_JUROR_OUTLIER", "int"),
  ("deltas.reporter_upheld", "DEFAULT_DELTAS_REPORTER_UPHELD", "int"),
  ("deltas.reporter_dismissed", "DEFAULT_DELTAS_REPORTER_DISMISSED", "int"),
  ("deltas.endorsement_created_sponsor", "DEFAULT_DELTAS_ENDORSEMENT_CREATED_SPONSOR", "int"),
  ("deltas.endorsement_created_sponsee", "DEFAULT_DELTAS_ENDORSEMENT_CREATED_SPONSEE", "int"),
  ("deltas.sponsor_liability_minor", "DEFAULT_DELTAS_SPONSOR_LIABILITY_MINOR", "int"),
  ("deltas.sponsor_liability_moderate", "DEFAULT_DELTAS_SPONSOR_LIABILITY_MODERATE", "int"),
  ("deltas.sponsor_liability_severe", "DEFAULT_DELTAS_SPONSOR_LIABILITY_SEVERE", "int"),
  ("liability.founder_multiplier", "DEFAULT_LIABILITY_FOUNDER_MULTIPLIER", "float"),
  ("liability.regular_multiplier", "DEFAULT_LIABILITY_REGULAR_MULTIPLIER", "float"),
  ("liability.sponsor_liability_floor", "DEFAULT_LIABILITY_SPONSOR_LIABILITY_FLOOR", "int"),
  ("report.base_weight", "DEFAULT_REPORT_BASE_WEIGHT", "float"),
  ("report.clamp_min", "DEFAULT_REPORT_CLAMP_MIN", "float"),
  ("report.clamp_max", "DEFAULT_REPORT_CLAMP_MAX", "float"),
  ("report.recency_half_life_hours", "DEFAULT_REPORT_RECENCY_HALF_LIFE_HOURS", "float"),
  ("report.case_threshold_micros", "DEFAULT_REPORT_CASE_THRESHOLD_MICROS", "int"),
  ("decay.positive_half_life_days", "DEFAULT_DECAY_POSITIVE_HALF_LIFE_DAYS", "int"),
  ("onboarding.default_membership_state", "DEFAULT_ONBOARDING_DEFAULT_MEMBERSHIP_STATE", "text"),
  ("onboarding.sponsor_gate_strategy", "DEFAULT_ONBOARDING_SPONSOR_GATE_STRATEGY", "text"),
  ("onboarding.sponsor_min_account_age_days", "DEFAULT_ONBOARDING_SPONSOR_MIN_ACCOUNT_AGE_DAYS", "int"),
  ("founder.max_founders_active", "DEFAULT_FOUNDER_MAX_FOUNDERS_ACTIVE", "int"),
  ("founder.max_expires_days", "DEFAULT_FOUNDER_MAX_EXPIRES_DAYS", "int"),
  ("founder.max_seed_delta", "DEFAULT_FOUNDER_MAX_SEED_DELTA", "int"),
  ("job.snapshot_interval_seconds", "DEFAULT_JOB_SNAPSHOT_INTERVAL_SECONDS", "int"),
  ("job.snapshot_batch_chunk_size", "DEFAULT_JOB_SNAPSHOT_BATCH_CHUNK_SIZE", "int"),
  // v1-AD-a additions (admin-dashboard-owned subset per PRD §5.2 minus
  // 10 sponsor-liability rows minus 1 rule_set.active_version_id)
  ("jury.severity_thresholds.minor", "DEFAULT_JURY_SEVERITY_THRESHOLDS_MINOR", "text"),
  ("jury.severity_thresholds.moderate", "DEFAULT_JURY_SEVERITY_THRESHOLDS_MODERATE", "text"),
  ("jury.severity_thresholds.severe", "DEFAULT_JURY_SEVERITY_THRESHOLDS_SEVERE", "text"),
  ("jury.diversity_constraints_enabled", "DEFAULT_JURY_DIVERSITY_CONSTRAINTS_ENABLED", "bool"),
  ("jury.appeal_panel_size_increase", "DEFAULT_JURY_APPEAL_PANEL_SIZE_INCREASE", "int"),
  ("jury.deadline_window_hours", "DEFAULT_JURY_DEADLINE_WINDOW_HOURS", "int"),
  ("decay.negative_half_life_days", "DEFAULT_DECAY_NEGATIVE_HALF_LIFE_DAYS", "int"),
  (
    "decay.endorsement_strength_half_life_days",
    "DEFAULT_DECAY_ENDORSEMENT_STRENGTH_HALF_LIFE_DAYS",
    "int",
  ),
  (
    "decay.jury_reliability_half_life_days",
    "DEFAULT_DECAY_JURY_RELIABILITY_HALF_LIFE_DAYS",
    "int",
  ),
  (
    "onboarding.sponsor_min_endorsement_strength",
    "DEFAULT_ONBOARDING_SPONSOR_MIN_ENDORSEMENT_STRENGTH",
    "int",
  ),
  (
    "onboarding.sponsor_allowlist_table_name",
    "DEFAULT_ONBOARDING_SPONSOR_ALLOWLIST_TABLE_NAME",
    "text",
  ),
  (
    "onboarding.provisional_membership_cooldown_days",
    "DEFAULT_ONBOARDING_PROVISIONAL_MEMBERSHIP_COOLDOWN_DAYS",
    "int",
  ),
  (
    "founder.founder_seal_visible_in_profile",
    "DEFAULT_FOUNDER_FOUNDER_SEAL_VISIBLE_IN_PROFILE",
    "bool",
  ),
  ("deltas.participation_weekly_active", "DEFAULT_DELTAS_PARTICIPATION_WEEKLY_ACTIVE", "int"),
  ("participation.dormancy_window_days", "DEFAULT_PARTICIPATION_DORMANCY_WINDOW_DAYS", "int"),
  ("deltas.participation_dormant", "DEFAULT_DELTAS_PARTICIPATION_DORMANT", "int"),
  ("participation.attestation_enabled", "DEFAULT_PARTICIPATION_ATTESTATION_ENABLED", "bool"),
  ("federation.inbound_advisory_only", "DEFAULT_FEDERATION_INBOUND_ADVISORY_ONLY", "bool"),
  ("federation.peer_attestation_ttl_days", "DEFAULT_FEDERATION_PEER_ATTESTATION_TTL_DAYS", "int"),
  ("federation.signature_required", "DEFAULT_FEDERATION_SIGNATURE_REQUIRED", "bool"),
  (
    "federation.quarantine_recommendation_severity_floor",
    "DEFAULT_FEDERATION_QUARANTINE_RECOMMENDATION_SEVERITY_FLOOR",
    "text",
  ),
  ("federation.outbound_publish_enabled", "DEFAULT_FEDERATION_OUTBOUND_PUBLISH_ENABLED", "bool"),
  ("rule_set.auto_carry_in_flight_cases", "DEFAULT_RULE_SET_AUTO_CARRY_IN_FLIGHT_CASES", "bool"),
  ("rule_set.text_max_bytes", "DEFAULT_RULE_SET_TEXT_MAX_BYTES", "int"),
  (
    "rule_set.version_propagation_delay_hours",
    "DEFAULT_RULE_SET_VERSION_PROPAGATION_DELAY_HOURS",
    "int",
  ),
  (
    "governance.dashboard.html_pages_enabled",
    "DEFAULT_GOVERNANCE_DASHBOARD_HTML_PAGES_ENABLED",
    "bool",
  ),
  (
    "governance.dashboard.step_up_enforced",
    "DEFAULT_GOVERNANCE_DASHBOARD_STEP_UP_ENFORCED",
    "bool",
  ),
  // v1-JM-a additions (v1 jury-mechanics sub-phase A — 27 new keys per
  // PRD §10 defaults matrix + §3.5 cascade). Distinct from the v1-AD-a
  // `jury.severity_thresholds.*` text display strings and the v1-AD-a
  // coarse `jury.diversity_constraints_enabled` toggle — both coexist.
  (
    "jury.panel_size.regular.minor",
    "DEFAULT_JURY_PANEL_SIZE_REGULAR_MINOR",
    "int",
  ),
  (
    "jury.panel_size.regular.moderate",
    "DEFAULT_JURY_PANEL_SIZE_REGULAR_MODERATE",
    "int",
  ),
  (
    "jury.panel_size.regular.severe",
    "DEFAULT_JURY_PANEL_SIZE_REGULAR_SEVERE",
    "int",
  ),
  (
    "jury.panel_size.founder.minor",
    "DEFAULT_JURY_PANEL_SIZE_FOUNDER_MINOR",
    "int",
  ),
  (
    "jury.panel_size.founder.moderate",
    "DEFAULT_JURY_PANEL_SIZE_FOUNDER_MODERATE",
    "int",
  ),
  (
    "jury.panel_size.founder.severe",
    "DEFAULT_JURY_PANEL_SIZE_FOUNDER_SEVERE",
    "int",
  ),
  (
    "jury.panel_size.probation.minor",
    "DEFAULT_JURY_PANEL_SIZE_PROBATION_MINOR",
    "int",
  ),
  (
    "jury.panel_size.probation.moderate",
    "DEFAULT_JURY_PANEL_SIZE_PROBATION_MODERATE",
    "int",
  ),
  (
    "jury.panel_size.probation.severe",
    "DEFAULT_JURY_PANEL_SIZE_PROBATION_SEVERE",
    "int",
  ),
  ("jury.quorum_fraction.minor", "DEFAULT_JURY_QUORUM_FRACTION_MINOR", "float"),
  (
    "jury.quorum_fraction.moderate",
    "DEFAULT_JURY_QUORUM_FRACTION_MODERATE",
    "float",
  ),
  ("jury.quorum_fraction.severe", "DEFAULT_JURY_QUORUM_FRACTION_SEVERE", "float"),
  (
    "jury.threshold_fraction.minor",
    "DEFAULT_JURY_THRESHOLD_FRACTION_MINOR",
    "float",
  ),
  (
    "jury.threshold_fraction.moderate",
    "DEFAULT_JURY_THRESHOLD_FRACTION_MODERATE",
    "float",
  ),
  (
    "jury.threshold_fraction.severe",
    "DEFAULT_JURY_THRESHOLD_FRACTION_SEVERE",
    "float",
  ),
  (
    "jury.constraints.no_majority_from_same_sponsor_cluster",
    "DEFAULT_JURY_CONSTRAINTS_NO_MAJORITY_FROM_SAME_SPONSOR_CLUSTER",
    "bool",
  ),
  (
    "jury.constraints.geographic_diversity_preferred",
    "DEFAULT_JURY_CONSTRAINTS_GEOGRAPHIC_DIVERSITY_PREFERRED",
    "bool",
  ),
  (
    "jury.constraints.no_recent_juror_repeat",
    "DEFAULT_JURY_CONSTRAINTS_NO_RECENT_JUROR_REPEAT",
    "bool",
  ),
  (
    "jury.constraints.juror_cooldown_days",
    "DEFAULT_JURY_CONSTRAINTS_JUROR_COOLDOWN_DAYS",
    "int",
  ),
  (
    "jury.constraints.no_same_endorsement_chain",
    "DEFAULT_JURY_CONSTRAINTS_NO_SAME_ENDORSEMENT_CHAIN",
    "bool",
  ),
  (
    "jury.constraints.max_retries_before_relax",
    "DEFAULT_JURY_CONSTRAINTS_MAX_RETRIES_BEFORE_RELAX",
    "int",
  ),
  (
    "jury.max_concurrent_assignments_per_juror_total",
    "DEFAULT_JURY_MAX_CONCURRENT_ASSIGNMENTS_PER_JUROR_TOTAL",
    "int",
  ),
  (
    "appeal.panel_size_multiplier",
    "DEFAULT_APPEAL_PANEL_SIZE_MULTIPLIER",
    "float",
  ),
  (
    "appeal.panel_size_floor_increment",
    "DEFAULT_APPEAL_PANEL_SIZE_FLOOR_INCREMENT",
    "int",
  ),
  ("appeal.threshold_tier_bump", "DEFAULT_APPEAL_THRESHOLD_TIER_BUMP", "int"),
  ("appeal.window_days", "DEFAULT_APPEAL_WINDOW_DAYS", "int"),
  (
    "appeal.auto_select_on_appeal_acceptance",
    "DEFAULT_APPEAL_AUTO_SELECT_ON_APPEAL_ACCEPTANCE",
    "bool",
  ),
  // v1-SL-a additions (sponsor-liability sub-phase A — 13 new keys per
  // PRD §10 defaults matrix; flat liability.* + job.grace_check_*
  // namespaces per PRD §18 B4 key-rename table).
  ("job.grace_check_batch_size", "DEFAULT_JOB_GRACE_CHECK_BATCH_SIZE", "int"),
  ("job.grace_check_interval_minutes", "DEFAULT_JOB_GRACE_CHECK_INTERVAL_MINUTES", "int"),
  ("job.grace_check_staleness_alert_multiplier", "DEFAULT_JOB_GRACE_CHECK_STALENESS_ALERT_MULTIPLIER", "float"),
  ("liability.grace_window_alert_threshold_hours", "DEFAULT_LIABILITY_GRACE_WINDOW_ALERT_THRESHOLD_HOURS", "int"),
  ("liability.grace_window_maximum_hours", "DEFAULT_LIABILITY_GRACE_WINDOW_MAXIMUM_HOURS", "int"),
  ("liability.grace_window_minimum_hours", "DEFAULT_LIABILITY_GRACE_WINDOW_MINIMUM_HOURS", "int"),
  ("liability.grace_window_minor_hours", "DEFAULT_LIABILITY_GRACE_WINDOW_MINOR_HOURS", "int"),
  ("liability.grace_window_moderate_hours", "DEFAULT_LIABILITY_GRACE_WINDOW_MODERATE_HOURS", "int"),
  ("liability.grace_window_severe_hours", "DEFAULT_LIABILITY_GRACE_WINDOW_SEVERE_HOURS", "int"),
  ("liability.multi_sponsor_escape_rule", "DEFAULT_LIABILITY_MULTI_SPONSOR_ESCAPE_RULE", "text"),
  ("liability.restoration_escapes_liability", "DEFAULT_LIABILITY_RESTORATION_ESCAPES_LIABILITY", "bool"),
  ("liability.restoration_severity_reduction_steps", "DEFAULT_LIABILITY_RESTORATION_SEVERITY_REDUCTION_STEPS", "int"),
  ("liability.revoke_rate_limit_per_day", "DEFAULT_LIABILITY_REVOKE_RATE_LIMIT_PER_DAY", "int"),
];

/// 34 after Perplexity-review 2026-04-17 added `job.snapshot_batch_chunk_size`
/// (GOTCHA-50f). The parity test asserts the `SEEDED_KEYS_WITH_CONSTS` length
/// matches this count.
pub const EXPECTED_SEED_COUNT: usize = 34;

/// v1-AD-a adds 27 admin-dashboard-owned keys to `SEEDED_KEYS_WITH_CONSTS`.
/// Parametric per advisor directive 2026-04-19 #4 — each subsequent v1
/// sub-PRD adds its own `EXPECTED_SEED_COUNT_V1_*` beside this one without
/// churning the v0 invariant. `rule_set.active_version_id` is deliberately
/// NOT counted here (absence-of-row is the "no active version" signal per
/// plan §4.1). Sponsor-liability v1's 10 `liability.*` keys ship in
/// sponsor-liability-v1 under their own parametric count.
pub const EXPECTED_SEED_COUNT_V1_AD: usize = 27;

/// v1-JM-a adds 27 jury-mechanics-owned keys to `SEEDED_KEYS_WITH_CONSTS`.
/// Parametric per advisor directive 2026-04-19 #4 — each v1 sub-PRD adds its
/// own `EXPECTED_SEED_COUNT_V1_*` beside the v0 + v1-AD-a invariants without
/// churning them. Count is authoritative against on-disk reality: plan §13
/// Task 8 reconciliation gate asserts `SEEDED_KEYS_WITH_CONSTS` contains
/// exactly this many v1-JM-a-block tuples AND the seed migration has exactly
/// this many INSERT rows.
pub const EXPECTED_SEED_COUNT_V1_JM: usize = 27;

/// v1-SL-a adds 13 sponsor-liability-owned keys to `SEEDED_KEYS_WITH_CONSTS`.
/// Parametric per advisor directive 2026-04-19 #4 — each v1 sub-PRD adds
/// its own `EXPECTED_SEED_COUNT_V1_*` beside the v0 + v1-AD-a + v1-JM-a
/// invariants without churning them. Count is authoritative against
/// on-disk reality: plan §13 Task 6 reconciliation gate asserts
/// `SEEDED_KEYS_WITH_CONSTS` contains exactly this many v1-SL-a-block
/// tuples AND the seed migration has exactly this many INSERT rows.
/// Per DQ #115: all `liability.grace_window_*_hours` keys are raw
/// integer hours (NOT micros-scaled); the lesson
/// `feedback_brehon_config_micros_scaled.md` scopes to reputation/score-
/// formula math, not wall-clock INTERVAL operands.
pub const EXPECTED_SEED_COUNT_V1_SL: usize = 13;

/// Enum variants for `federation.quarantine_recommendation_severity_floor`.
const ENUM_SEVERITY_FLOOR: &[&str] = &["minor", "moderate", "severe"];

/// Enum variants for `jury.severity_thresholds.*` — expressed as "majority"
/// or a "<percentage>%" literal. v1 kept as free text because there's no
/// closed set yet (`majority` is a synonym for `>50%`).
const ENUM_SEVERITY_THRESHOLD: &[&str] = &["majority", "55%", "60%", "66%", "75%", "unanimous"];

/// Enum variants for `onboarding.default_membership_state` (v0 key). Listed
/// here so the metadata entry stays self-contained.
const ENUM_MEMBERSHIP_STATE: &[&str] = &["member", "provisional", "suspended"];

/// Enum variants for `onboarding.sponsor_gate_strategy` (v0 key widened in
/// v1-AD per OQ-020: `age` | `reputation` | `allowlist`). Listed here so
/// v1-AD-b's POST /admin/config handler has one canonical reference.
const ENUM_SPONSOR_GATE_STRATEGY: &[&str] = &["age", "reputation", "allowlist"];

/// Enum variants for `liability.multi_sponsor_escape_rule` (v1-SL-a key).
/// Per PRD §13.1 + §13 escape-rule table: `any_revocation` (default) means
/// one sponsor revoking escapes all; `all_revocation` requires every active
/// sponsor to revoke; `majority_revocation` requires >50%.
const ENUM_MULTI_SPONSOR_ESCAPE_RULE: &[&str] =
  &["any_revocation", "all_revocation", "majority_revocation"];

/// Compile-time metadata for every seeded `governance_config` key. Length
/// must equal `SEEDED_KEYS_WITH_CONSTS.len()` (enforced by
/// `parity::every_seeded_key_has_metadata`). Order is not significant —
/// the parity test matches by `key` name, not index.
pub const CONFIG_KEY_METADATA: &[ConfigKeyMetadata] = &[
  // ---- v0 keys (34) ------------------------------------------------------
  ConfigKeyMetadata {
    key: "thresholds.jury_reliability",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 100.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Jury-eligibility capability cutoff (reputation_snapshot jury_reliability).",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "thresholds.reporting_accuracy",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 100.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Reporter capability cutoff.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "thresholds.endorsement_strength",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 100.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Sponsor capability cutoff.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "jury.panel_size",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 3.0, max: 21.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Default panel size at jury seating.",
    doc_anchor: "01#5.6",
  },
  ConfigKeyMetadata {
    key: "jury.quorum",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 21.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Minimum votes required for a quorum.",
    doc_anchor: "01#5.6",
  },
  ConfigKeyMetadata {
    key: "jury.age_requirement_days",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 3650.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Minimum account age (days) for jury eligibility.",
    doc_anchor: "01#5.6",
  },
  ConfigKeyMetadata {
    key: "jury.max_concurrent_assignments",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 20.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Max open jury assignments a single juror can hold.",
    doc_anchor: "01#5.6",
  },
  ConfigKeyMetadata {
    key: "jury.fallback_on_small_pool",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "If true, allow reduced panel when pool too small.",
    doc_anchor: "01#5.6",
  },
  ConfigKeyMetadata {
    key: "deltas.juror_aligned",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: -1000.0, max: 1000.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Reputation delta for majority-aligned juror.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "deltas.juror_outlier",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: -1000.0, max: 1000.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Reputation delta for outlier juror.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "deltas.reporter_upheld",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: -1000.0, max: 1000.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Reputation delta when a report is upheld.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "deltas.reporter_dismissed",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: -1000.0, max: 1000.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Reputation delta when a report is dismissed.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "deltas.endorsement_created_sponsor",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: -1000.0, max: 1000.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Reputation delta granted to sponsor on endorsement.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "deltas.endorsement_created_sponsee",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: -1000.0, max: 1000.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Reputation delta granted to sponsee on endorsement.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "deltas.sponsor_liability_minor",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: -10000.0, max: 0.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Reputation delta applied to sponsors on minor sanction.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "deltas.sponsor_liability_moderate",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: -10000.0, max: 0.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Reputation delta applied to sponsors on moderate sanction.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "deltas.sponsor_liability_severe",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: -10000.0, max: 0.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Reputation delta applied to sponsors on severe sanction.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "liability.founder_multiplier",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 0.0, max: 10.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Liability multiplier applied to founders.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "liability.regular_multiplier",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 0.0, max: 10.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Liability multiplier applied to non-founders.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "liability.sponsor_liability_floor",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: -100000.0, max: 0.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Floor for sponsor liability clamping (OQ-024).",
    doc_anchor: "99#OQ-024",
  },
  ConfigKeyMetadata {
    key: "report.base_weight",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 0.0, max: 10.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Base weight for a fresh report before modifiers.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "report.clamp_min",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 0.0, max: 10.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Minimum weight clamp for a report.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "report.clamp_max",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 0.0, max: 10.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Maximum weight clamp for a report.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "report.recency_half_life_hours",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 1.0, max: 8760.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Half-life (hours) used in report recency weighting.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "report.case_threshold_micros",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 100_000_000.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Accumulated report score (micros) required to open a case.",
    doc_anchor: "01#5.2",
  },
  ConfigKeyMetadata {
    key: "decay.positive_half_life_days",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 3650.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextSnapshotJob,
    description: "Positive-delta decay half-life (days) in reputation snapshots.",
    doc_anchor: "01#5.3",
  },
  ConfigKeyMetadata {
    key: "onboarding.default_membership_state",
    value_type: ValueType::Enum,
    valid_range: None,
    valid_enum: Some(ENUM_MEMBERSHIP_STATE),
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Default membership state at user registration.",
    doc_anchor: "01#5.4",
  },
  ConfigKeyMetadata {
    key: "onboarding.sponsor_gate_strategy",
    value_type: ValueType::Enum,
    valid_range: None,
    valid_enum: Some(ENUM_SPONSOR_GATE_STRATEGY),
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Strategy used to gate sponsor endorsement (age/reputation/allowlist).",
    doc_anchor: "99#OQ-020",
  },
  ConfigKeyMetadata {
    key: "onboarding.sponsor_min_account_age_days",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 3650.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Min account age (days) for sponsor endorsement under 'age' gate.",
    doc_anchor: "99#OQ-020",
  },
  ConfigKeyMetadata {
    key: "founder.max_founders_active",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 10_000.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Maximum simultaneously-active founders per instance.",
    doc_anchor: "01#5.5",
  },
  ConfigKeyMetadata {
    key: "founder.max_expires_days",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 3650.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Max lifetime (days) of a founder seed.",
    doc_anchor: "01#5.5",
  },
  ConfigKeyMetadata {
    key: "founder.max_seed_delta",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 10_000.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Max reputation delta applied to a founder at seeding.",
    doc_anchor: "01#5.5",
  },
  ConfigKeyMetadata {
    key: "job.snapshot_interval_seconds",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 60.0, max: 86_400.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextSnapshotJob,
    description: "Cadence (seconds) of the reputation-snapshot job.",
    doc_anchor: "01#5.3",
  },
  ConfigKeyMetadata {
    key: "job.snapshot_batch_chunk_size",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 10_000.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextSnapshotJob,
    description: "Chunk size for snapshot-job batching.",
    doc_anchor: "01#5.3",
  },
  // ---- v1-AD-a additions (27) -------------------------------------------
  ConfigKeyMetadata {
    key: "jury.severity_thresholds.minor",
    value_type: ValueType::Enum,
    valid_range: None,
    valid_enum: Some(ENUM_SEVERITY_THRESHOLD),
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Quorum threshold for minor-severity cases.",
    doc_anchor: "01#5.6",
  },
  ConfigKeyMetadata {
    key: "jury.severity_thresholds.moderate",
    value_type: ValueType::Enum,
    valid_range: None,
    valid_enum: Some(ENUM_SEVERITY_THRESHOLD),
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Quorum threshold for moderate-severity cases.",
    doc_anchor: "01#5.6",
  },
  ConfigKeyMetadata {
    key: "jury.severity_thresholds.severe",
    value_type: ValueType::Enum,
    valid_range: None,
    valid_enum: Some(ENUM_SEVERITY_THRESHOLD),
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Quorum threshold for severe-severity cases.",
    doc_anchor: "01#5.6",
  },
  ConfigKeyMetadata {
    key: "jury.diversity_constraints_enabled",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Whether diversity constraints apply to jury seating.",
    doc_anchor: "01#5.6",
  },
  ConfigKeyMetadata {
    key: "jury.appeal_panel_size_increase",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 20.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Extra jurors added to appeal panels beyond the base panel size.",
    doc_anchor: "01#5.7",
  },
  ConfigKeyMetadata {
    key: "jury.deadline_window_hours",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 720.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Deadline window (hours) for jurors to vote.",
    doc_anchor: "04#4.2",
  },
  ConfigKeyMetadata {
    key: "decay.negative_half_life_days",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 3650.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextSnapshotJob,
    description: "Negative-delta decay half-life (days).",
    doc_anchor: "01#5.3",
  },
  ConfigKeyMetadata {
    key: "decay.endorsement_strength_half_life_days",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 3650.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextSnapshotJob,
    description: "Endorsement-strength decay half-life (days).",
    doc_anchor: "01#5.3",
  },
  ConfigKeyMetadata {
    key: "decay.jury_reliability_half_life_days",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 3650.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextSnapshotJob,
    description: "Jury-reliability decay half-life (days).",
    doc_anchor: "01#5.3",
  },
  ConfigKeyMetadata {
    key: "onboarding.sponsor_min_endorsement_strength",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 100.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Min endorsement_strength to sponsor under the 'reputation' gate.",
    doc_anchor: "99#OQ-020",
  },
  ConfigKeyMetadata {
    key: "onboarding.sponsor_allowlist_table_name",
    value_type: ValueType::Text,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Table name the 'allowlist' sponsor gate reads from.",
    doc_anchor: "99#OQ-020",
  },
  ConfigKeyMetadata {
    key: "onboarding.provisional_membership_cooldown_days",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 3650.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Cooldown (days) before a provisional member promotes to full member.",
    doc_anchor: "99#OQ-016",
  },
  ConfigKeyMetadata {
    key: "founder.founder_seal_visible_in_profile",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Whether the founder seal is visible in public profiles.",
    doc_anchor: "99#OQ-017",
  },
  ConfigKeyMetadata {
    key: "deltas.participation_weekly_active",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: -1000.0, max: 1000.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextSnapshotJob,
    description: "Reputation delta for weekly-active participation.",
    doc_anchor: "99#OQ-019",
  },
  ConfigKeyMetadata {
    key: "participation.dormancy_window_days",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 3650.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextSnapshotJob,
    description: "Days of inactivity before an account is classified dormant.",
    doc_anchor: "99#OQ-019",
  },
  ConfigKeyMetadata {
    key: "deltas.participation_dormant",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: -1000.0, max: 1000.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextSnapshotJob,
    description: "Reputation delta applied when an account turns dormant.",
    doc_anchor: "99#OQ-019",
  },
  ConfigKeyMetadata {
    key: "participation.attestation_enabled",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Whether admin attestation of participation is enabled (decide-later).",
    doc_anchor: "99#OQ-V1-AD-04",
  },
  ConfigKeyMetadata {
    key: "federation.inbound_advisory_only",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "When true, inbound federation signals are advisory (no enforcement).",
    doc_anchor: "99#ADR-006",
  },
  ConfigKeyMetadata {
    key: "federation.peer_attestation_ttl_days",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 3650.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "TTL (days) for peer-attestation freshness.",
    doc_anchor: "07",
  },
  ConfigKeyMetadata {
    key: "federation.signature_required",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "When true, reject inbound governance activities missing a valid signature.",
    doc_anchor: "06",
  },
  ConfigKeyMetadata {
    key: "federation.quarantine_recommendation_severity_floor",
    value_type: ValueType::Enum,
    valid_range: None,
    valid_enum: Some(ENUM_SEVERITY_FLOOR),
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Minimum severity for inbound quarantine recommendations to be surfaced.",
    doc_anchor: "07",
  },
  ConfigKeyMetadata {
    key: "federation.outbound_publish_enabled",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Whether to publish outbound governance activities.",
    doc_anchor: "07",
  },
  ConfigKeyMetadata {
    key: "rule_set.auto_carry_in_flight_cases",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "When true, in-flight cases keep their snapshotted rule-set version.",
    doc_anchor: "99#OQ-002",
  },
  ConfigKeyMetadata {
    key: "rule_set.text_max_bytes",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1_024.0, max: 1_048_576.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Upper bound (bytes) on rule-set Markdown text size.",
    doc_anchor: "99#OQ-002",
  },
  ConfigKeyMetadata {
    key: "rule_set.version_propagation_delay_hours",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 720.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Hours between rule-set activation and when new cases use it.",
    doc_anchor: "99#OQ-018",
  },
  ConfigKeyMetadata {
    key: "governance.dashboard.html_pages_enabled",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Whether the admin dashboard serves server-rendered HTML pages.",
    doc_anchor: "§6.1",
  },
  ConfigKeyMetadata {
    key: "governance.dashboard.step_up_enforced",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Whether step-up auth is enforced (v2); v1 default is advisory.",
    doc_anchor: "§7.2",
  },
  // ---- v1-JM-a additions (27) -------------------------------------------
  // jury.panel_size.<status>.<severity> — 9 keys (int, range 3..=11 per PRD
  // §3.4). NextJuryCycle: in-flight juries keep their snapshotted panel size
  // via moderation_case.panel_size_snapshot (ADR-010).
  ConfigKeyMetadata {
    key: "jury.panel_size.regular.minor",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 3.0, max: 11.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Panel size for regular-status targets with minor-severity cases.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.panel_size.regular.moderate",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 3.0, max: 11.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Panel size for regular-status targets with moderate-severity cases.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.panel_size.regular.severe",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 3.0, max: 11.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Panel size for regular-status targets with severe-severity cases.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.panel_size.founder.minor",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 3.0, max: 11.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Panel size for founder-status targets with minor-severity cases.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.panel_size.founder.moderate",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 3.0, max: 11.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Panel size for founder-status targets with moderate-severity cases.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.panel_size.founder.severe",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 3.0, max: 11.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Panel size for founder-status targets with severe-severity cases.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.panel_size.probation.minor",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 3.0, max: 11.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Panel size for probation-status targets with minor-severity cases.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.panel_size.probation.moderate",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 3.0, max: 11.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Panel size for probation-status targets with moderate-severity cases.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.panel_size.probation.severe",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 3.0, max: 11.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Panel size for probation-status targets with severe-severity cases.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  // jury.quorum_fraction.<severity> — 3 keys (float). NextJuryCycle.
  ConfigKeyMetadata {
    key: "jury.quorum_fraction.minor",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 0.5, max: 1.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Fraction of seated jurors required to reach quorum (minor severity).",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.quorum_fraction.moderate",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 0.5, max: 1.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Fraction of seated jurors required to reach quorum (moderate severity).",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.quorum_fraction.severe",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 0.5, max: 1.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Fraction of seated jurors required to reach quorum (severe severity).",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  // jury.threshold_fraction.<severity> — 3 keys (float). NextJuryCycle.
  ConfigKeyMetadata {
    key: "jury.threshold_fraction.minor",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 0.5001, max: 1.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Fraction of votes required to uphold (minor severity; strict majority).",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.threshold_fraction.moderate",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 0.5001, max: 1.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Fraction of votes required to uphold (moderate severity).",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.threshold_fraction.severe",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 0.5001, max: 1.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Fraction of votes required to uphold (severe severity; supermajority).",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  // jury.constraints.* — 5 keys (4 bool + 1 int). NextJuryCycle.
  ConfigKeyMetadata {
    key: "jury.constraints.no_majority_from_same_sponsor_cluster",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Granular constraint: forbid a majority of jurors from one sponsor cluster.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.constraints.geographic_diversity_preferred",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Granular constraint: prefer geographic diversity on panel pick.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.constraints.no_recent_juror_repeat",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Granular constraint: exclude jurors with recent service (see cooldown key).",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.constraints.juror_cooldown_days",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 365.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Days since last juror service before a person is re-eligible.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.constraints.no_same_endorsement_chain",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: true,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Granular constraint: forbid jurors in a shared endorsement chain (v1.5 gate).",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  // jury.constraints.max_retries_before_relax + jury.max_concurrent_* — 2 keys.
  ConfigKeyMetadata {
    key: "jury.constraints.max_retries_before_relax",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 50.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Retry attempts before select_eligible_jurors relaxes constraints.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "jury.max_concurrent_assignments_per_juror_total",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 20.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Maximum concurrent jury assignments per juror across all cases.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  // appeal.* — 5 keys. window_days + auto_select_on_appeal_acceptance apply
  // immediately (LIVE read per PRD §9.1 step 9); the rest NextJuryCycle.
  ConfigKeyMetadata {
    key: "appeal.panel_size_multiplier",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 1.0, max: 3.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Multiplier on original panel size when seating an appeal panel.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "appeal.panel_size_floor_increment",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 10.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Minimum extra jurors added to appeal panels beyond the multiplied base.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "appeal.threshold_tier_bump",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 2.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::NextJuryCycle,
    description: "Severity-tier bump when moving from original panel to appeal panel.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "appeal.window_days",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 90.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Days after case-decision during which an appeal can be requested.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "appeal.auto_select_on_appeal_acceptance",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Whether accepting an appeal request auto-seats the appeal panel.",
    doc_anchor: "v1-jury-mechanics.prd.md§10",
  },
  // ---- v1-SL-a keys (13) -------------------------------------------------
  // liability.grace_window_*_hours — 6 keys (int; raw hours per DQ #115)
  ConfigKeyMetadata {
    key: "liability.grace_window_minor_hours",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 720.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Hours before sponsor liability fires for a minor-severity decision (community-configurable grace window).",
    doc_anchor: "v1-sponsor-liability.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "liability.grace_window_moderate_hours",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 720.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Hours before sponsor liability fires for a moderate-severity decision (community-configurable grace window).",
    doc_anchor: "v1-sponsor-liability.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "liability.grace_window_severe_hours",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 720.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Hours before sponsor liability fires for a severe-severity decision (community-configurable grace window).",
    doc_anchor: "v1-sponsor-liability.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "liability.grace_window_minimum_hours",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 720.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Instance-level floor: communities cannot set any per-tier grace window below this value.",
    doc_anchor: "v1-sponsor-liability.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "liability.grace_window_maximum_hours",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 720.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Instance-level ceiling: communities cannot set any per-tier grace window above this value.",
    doc_anchor: "v1-sponsor-liability.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "liability.grace_window_alert_threshold_hours",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 720.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Instance-level audit alert threshold: admin dashboard warns when a community sets any grace window below this value.",
    doc_anchor: "v1-sponsor-liability.prd.md§10",
  },
  // liability.restoration_* — 2 keys (1 bool + 1 int)
  ConfigKeyMetadata {
    key: "liability.restoration_escapes_liability",
    value_type: ValueType::Bool,
    valid_range: None,
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Whether a sponsee's restoration action within the grace window escapes sponsor liability entirely.",
    doc_anchor: "v1-sponsor-liability.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "liability.restoration_severity_reduction_steps",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 0.0, max: 3.0 }),
    valid_enum: None,
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Number of severity steps reduced when restoration escapes liability (0 = full escape; 1+ = partial mitigation cascade).",
    doc_anchor: "v1-sponsor-liability.prd.md§10",
  },
  // liability.multi_sponsor_escape_rule — 1 key (text/enum)
  ConfigKeyMetadata {
    key: "liability.multi_sponsor_escape_rule",
    value_type: ValueType::Enum,
    valid_range: None,
    valid_enum: Some(ENUM_MULTI_SPONSOR_ESCAPE_RULE),
    scope: ConfigScope::Both,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Escape rule when a sponsee has multiple sponsors: any_revocation, all_revocation, or majority_revocation.",
    doc_anchor: "v1-sponsor-liability.prd.md§10",
  },
  // liability.revoke_rate_limit_per_day — 1 key (int)
  ConfigKeyMetadata {
    key: "liability.revoke_rate_limit_per_day",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 100.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Maximum sponsor revocations permitted per user per rolling 24-hour window (instance-level cap).",
    doc_anchor: "v1-sponsor-liability.prd.md§10",
  },
  // job.grace_check_* — 3 keys (2 int + 1 float)
  ConfigKeyMetadata {
    key: "job.grace_check_interval_minutes",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 60.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "How often (minutes) the grace-window scheduler checks for expiring windows; config changes take effect at next server restart.",
    doc_anchor: "v1-sponsor-liability.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "job.grace_check_batch_size",
    value_type: ValueType::Int,
    valid_range: Some(NumericRange { min: 1.0, max: 10000.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Maximum sponsee records processed per grace-check scheduler tick.",
    doc_anchor: "v1-sponsor-liability.prd.md§10",
  },
  ConfigKeyMetadata {
    key: "job.grace_check_staleness_alert_multiplier",
    value_type: ValueType::Float,
    valid_range: Some(NumericRange { min: 1.0, max: 10.0 }),
    valid_enum: None,
    scope: ConfigScope::Instance,
    requires_re_jury: false,
    requires_step_up: false,
    apply_at_default: ApplyAt::Immediate,
    description: "Multiplier of interval_minutes before a stale grace-check run triggers a staleness alert.",
    doc_anchor: "v1-sponsor-liability.prd.md§10",
  },
];


#[cfg(test)]
mod parity {
  use super::*;

  #[test]
  fn seeded_keys_count_matches_const_count() {
    let expected = EXPECTED_SEED_COUNT
      + EXPECTED_SEED_COUNT_V1_AD
      + EXPECTED_SEED_COUNT_V1_JM
      + EXPECTED_SEED_COUNT_V1_SL;
    assert_eq!(
      SEEDED_KEYS_WITH_CONSTS.len(),
      expected,
      "SEEDED_KEYS_WITH_CONSTS length ({}) must equal EXPECTED_SEED_COUNT ({}) + \
       EXPECTED_SEED_COUNT_V1_AD ({}) + EXPECTED_SEED_COUNT_V1_JM ({}) + \
       EXPECTED_SEED_COUNT_V1_SL ({}) = {} — add/remove keys in both places \
       when changing the seed list",
      SEEDED_KEYS_WITH_CONSTS.len(),
      EXPECTED_SEED_COUNT,
      EXPECTED_SEED_COUNT_V1_AD,
      EXPECTED_SEED_COUNT_V1_JM,
      EXPECTED_SEED_COUNT_V1_SL,
      expected,
    );
  }

  /// Every seeded key must appear in `CONFIG_KEY_METADATA` exactly once, and
  /// vice versa. v1-AD-a invariant (NOT4 2026-04-19): no DB-only key (reject
  /// additions to seed SQL without matching Rust metadata) and no metadata-
  /// only key (reject `CONFIG_KEY_METADATA` entries without matching seed).
  #[test]
  fn every_seeded_key_has_metadata() {
    assert_eq!(
      CONFIG_KEY_METADATA.len(),
      SEEDED_KEYS_WITH_CONSTS.len(),
      "CONFIG_KEY_METADATA length ({}) must equal SEEDED_KEYS_WITH_CONSTS length ({})",
      CONFIG_KEY_METADATA.len(),
      SEEDED_KEYS_WITH_CONSTS.len(),
    );
    for (key, _const_name, vtype) in SEEDED_KEYS_WITH_CONSTS {
      let md = CONFIG_KEY_METADATA.iter().find(|m| m.key == *key).unwrap_or_else(|| {
        panic!("seeded key `{key}` has no matching CONFIG_KEY_METADATA entry")
      });
      let vt_matches = match *vtype {
        "int" => matches!(md.value_type, ValueType::Int),
        "float" => matches!(md.value_type, ValueType::Float),
        "bool" => matches!(md.value_type, ValueType::Bool),
        "text" => matches!(md.value_type, ValueType::Text | ValueType::Enum),
        other => panic!("unknown value_type '{other}' for key '{key}'"),
      };
      assert!(
        vt_matches,
        "seeded key `{key}` has type `{vtype}` but CONFIG_KEY_METADATA declares {:?}",
        md.value_type
      );
    }
  }

  /// Ensure every seeded key has a matching `const_default_*` fallback that
  /// returns `Some`. Catches the class of bug where the seed list and the
  /// const-default lookup tables drift.
  #[test]
  fn every_seeded_key_has_const_fallback() {
    for (key, _const_name, vtype) in SEEDED_KEYS_WITH_CONSTS {
      let resolved = match *vtype {
        "int" => const_default_int(key).is_some(),
        "float" => const_default_float(key).is_some(),
        "bool" => const_default_bool(key).is_some(),
        "text" => const_default_text(key).is_some(),
        other => panic!("unknown value_type '{other}' for key '{key}'"),
      };
      assert!(
        resolved,
        "seeded key `{key}` (type {vtype}) has no matching Rust const fallback — add it to \
         const_default_{vtype}"
      );
    }
  }

  /// `rule_set.active_version_id` is deliberately NOT seeded and has NO Rust
  /// const default (v1-AD-a advisor edit #2). Absence-of-row is the "no active
  /// version" signal — v1-AD-c's rule-set handlers read this via the
  /// `get_int_opt` accessor landing alongside this test. Regressing any of
  /// these three invariants re-introduces the footgun advisor edit #2 removed.
  #[test]
  fn rule_set_active_version_not_in_seeded_keys() {
    assert!(
      !SEEDED_KEYS_WITH_CONSTS
        .iter()
        .any(|(k, _, _)| *k == "rule_set.active_version_id"),
      "rule_set.active_version_id must NOT be in SEEDED_KEYS_WITH_CONSTS \
       (v1-AD-a advisor edit #2)"
    );
    assert!(
      !CONFIG_KEY_METADATA
        .iter()
        .any(|m| m.key == "rule_set.active_version_id"),
      "rule_set.active_version_id must NOT be in CONFIG_KEY_METADATA \
       (v1-AD-a advisor edit #2)"
    );
    assert!(
      const_default_int("rule_set.active_version_id").is_none(),
      "rule_set.active_version_id must have NO Rust const default"
    );
  }
}
