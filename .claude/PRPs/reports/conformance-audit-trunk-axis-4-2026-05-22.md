---
axis: 4
scope: trunk (governance-v0 @ 7e6c4202f)
date: 2026-05-22
mode: file (all governance files in three roots)
---

# Conformance audit — axis 4: Error idiom at trust boundary

## Scope

Three federation/governance roots on `governance-v0` @ `7e6c4202f`:

- `crates/apub/activities/src/governance/` (5 files)
- `crates/api/api/src/governance/` (31 files)
- `crates/db_schema/src/source/governance/` (26 files)

Total: 62 governance .rs files.

## Detection

Per `axes/4-error-idiom.md`:

1. Grepped `\.unwrap_or_default()` across all three roots.
2. Grepped `\.unwrap_or("` (string literal default) across all three roots.
3. Grepped `\.unwrap_or_else(String::new)` across all three roots.
4. Grepped `\.unwrap_or` (all variants) across all three roots to catch
   numeric and non-string-literal patterns for completeness.

## Inventory of hits by file

### `crates/apub/activities/src/governance/` (5 files)

#### `publish_trust_attestation.rs`

**Line 351–352** — `synthesise_object_id` function:
```rust
fn synthesise_object_id(
  activity_id: &Url,
  attestation_type: AttestationType,
) -> Result<Url, url::ParseError> {
  let id = format!(
    "{}/object/{}",
    activity_id.as_str(),
    serde_json::to_value(attestation_type)
      .map(|v| v.as_str().unwrap_or("unknown").to_string())
      .unwrap_or_else(|_| "unknown".to_string()),
  );
  Url::parse(&id)
}
```

Context: `attestation_type: AttestationType` is a locally-constructed enum
value passed by the calling code within the same binary — NOT a field
from a remote actor or deserialized activity. `serde_json::to_value(local_enum)`
serializes a Rust enum to extract a URL-fragment component; `v.as_str().unwrap_or("unknown")`
handles the case where the JSON value is not a string (defensive, but the
resulting URL fragment is non-sensitive — a malformed fragment produces a
well-formed but differently-shaped URL, not a trust-boundary integrity
violation). The `synthesise_object_id` function returns `Result<Url, url::ParseError>`,
so callers propagate errors via `?` anyway if URL construction fails.

Classification: **Tier 3** — URL-fragment composition from a local enum;
not a trust-boundary remote-actor field.

Same-file siblings that DO use `.ok_or_else(|| LemmyErrorType::...)?`:
- Line 65–67: `object_actor` extraction from untyped AP object
- Lines 105–110: `actor_domain()` trait impl
- Lines 131–133: `subject_url` extraction from untyped object stub
- Line 152: `val.ok_or_else(...)` on DB config read

These siblings confirm the file correctly uses the hard-error pattern at
actual trust boundaries. The `.unwrap_or("unknown")` at line 351 is
deliberately weaker for a non-trust-boundary use case.

**Line 161** — `unwrap_or_else(std::sync::PoisonError::into_inner)`:
Standard Mutex poison recovery; not flaggable under axis 4.

#### `inbox.rs`

**Line 525** — `i64::try_from(activity.payload_size_bytes()?).unwrap_or(i64::MAX)`:
Numeric type conversion fallback for a locally-computed byte count. If
`usize` exceeds `i64::MAX` (impossible on 64-bit platforms), falls back
to `i64::MAX` which causes an over-limit rejection — a safe conservative
direction. Not a remote-actor string field.

**Line 547** — `unwrap_or_else(std::sync::PoisonError::into_inner)`:
Standard Mutex poison recovery; not flaggable.

**No `.unwrap_or_default()` found in `inbox.rs`.** The Phase-6 Finding 6.1
(`receive_remote_moderation_label` domain extraction) was closed at fix-impl-3
SHA `8b04e69a6`. The fix is confirmed present — no residual `.unwrap_or_default()`
on `.domain()` anywhere in this file.

### `crates/api/api/src/governance/` (31 files)

#### `audit_projection.rs`

**Lines 28, 33, 38, 49** — four `.unwrap_or("")` inside `project_to_audit_entry`:

```rust
let scope = payload.get("scope").and_then(|v| v.as_str()).unwrap_or("").to_string();
let key   = payload.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
let value_type = payload.get("value_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
// ... (line 40: unwrap_or(Value::Null) on a serde Value — not a string default)
let reason = payload.get("reason").and_then(|v| v.as_str()).unwrap_or("").to_string();
```

Context: `project_to_audit_entry(row: GovernanceLog)` reads the
`payload: Value` JSONB column of a `GovernanceLog` row — a column that
is exclusively written by local code paths (Postgres trigger-managed
hash chain; `GovernanceLogInsertForm` insert path). The file-level doc
comment (lines 12–17) makes this explicit:

> "Unknown / missing payload fields degrade to empty strings / `Value::Null`
> — we never fail a whole audit page on a single malformed legacy row (shell-
> wrapper rows predating this handler always produce well-formed payloads, but
> future migrations may add fields and older rows should still list)."

This is an intentional degradation policy for an admin UI projection.
`GovernanceLog` is not a remote-actor data type; it is the instance's own
hash-chain log. There is no remote actor in scope.

Classification: **Tier 3** — locally-written JSONB projection with an
explicit intentional degradation policy for legacy-schema backward compat.

#### All other api governance files

The remaining 30 files produce only:
- Numeric conversions: `i64::try_from(pool.len()).unwrap_or(i64::MAX)`,
  `i32::try_from(panel_size).unwrap_or(i32::MAX)`, `usize::try_from(matches).unwrap_or(0)`
- Pagination defaults: `data.page.unwrap_or(DEFAULT_PAGE)`,
  `data.limit.unwrap_or(DEFAULT_LIMIT)`
- Optional-field defaults from local user request data (API inputs): `data.dry_run.unwrap_or(false)`,
  `cooldown_days.unwrap_or(0)`, `endorsement_id.map(i64::from).unwrap_or(0)`
- Local algorithm parameters: `watermark.unwrap_or_else(|| DateTime::from_timestamp(...))`,
  `reason_code_json = serde_json::to_value(...).unwrap_or(Value::Null)`

None of these are remote-actor fields. All numeric-conversion `.unwrap_or(MAX)`
patterns are conservative (saturate-high produces a deliberate over-limit
rejection, not a silent empty-string persistence).

### `crates/db_schema/src/source/governance/` (26 files)

**`federation_peer.rs:66`** — `result.unwrap_or(FederationPeerTrust::Unknown)`:
DB lookup result fallback returning an enum variant. Not a string-field
trust-boundary access.

No other `.unwrap_or` variants found.

## Findings

**Tier 1:** 0
**Tier 2:** 0
**Tier 3 (informational):** 5

| # | File | Lines | Pattern | Classification | Rationale |
|---|---|---|---|---|---|
| T3-1 | `publish_trust_attestation.rs` | 351–352 | `.unwrap_or("unknown")` on `serde_json::to_value(local_enum).as_str()` | Tier 3 | Local enum URL-fragment composition; not a remote-actor field |
| T3-2 | `audit_projection.rs` | 28, 33, 38, 49 | `.unwrap_or("")` on `GovernanceLog.payload` JSONB fields (4 sites) | Tier 3 | Locally-written log column; intentional degradation policy documented in-file |

## Phase-6 Finding 6.1 verification

The triggering defect (`receive_remote_moderation_label` domain extraction
using `.domain().map(str::to_string).unwrap_or_default()`) is **not present
on trunk**. No `.unwrap_or_default()` exists anywhere in the three governance
roots. The fix at SHA `8b04e69a6` is confirmed.

## Hypothesis discipline

All flags are hypotheses until the compiler confirms them (post-§15) or a
sibling diff shows new code is strictly weaker than an enforced contract.

For T3-1 (`publish_trust_attestation.rs:351`): the same-file siblings at
lines 65, 105–110, 131, 152 all use `.ok_or_else(|| LemmyErrorType::...)?`
at the actual trust boundary (remote AP object field extraction,
`actor_domain()` trait, per-actor rate-limit config). The `.unwrap_or("unknown")`
at line 351 is for a local enum serialization step — the divergence from the
sibling pattern is intentional and contextually appropriate (Tier-3 per axis-4 spec).

For T3-2 (`audit_projection.rs:28,33,38,49`): no sibling in the api governance
root uses `.ok_or_else()` for JSONB payload field access, confirming this file
does not participate in the trust-boundary error contract. Intentional degradation
policy is documented in-file.

## Evidence strings

```
axis-4: publish_trust_attestation.rs:351 pattern=.unwrap_or("unknown") on serde_json enum value context=local-enum-url-fragment NOT-trust-boundary tier=3
axis-4: audit_projection.rs:28,33,38,49 pattern=.unwrap_or("") on GovernanceLog.payload JSONB context=local-log-column-intentional-degradation NOT-trust-boundary tier=3
```
