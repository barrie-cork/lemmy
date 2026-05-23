---
axis: 3
scope: trunk (governance-v0 @ 7e6c4202f)
date: 2026-05-22
mode: file (all governance files in three roots)
---

# Conformance audit — axis 3: Trait-bound completeness

## Scope

Three federation/governance roots on `governance-v0` @ `7e6c4202f`:

- `crates/apub/activities/src/governance/` (5 files)
- `crates/api/api/src/governance/` (31 files)
- `crates/db_schema/src/source/governance/` (26 files)

Total: 62 governance .rs files.

## Detection

Per `axes/3-trait-bound.md`:

1. Grepped for `#[async_trait]` usages across all three roots.
2. Grepped for `fn.*<[A-Z].*:.*>` (generic functions) across all three roots.
3. For each generic function / trait, read the where clause and compared to in-file
   and in-crate siblings.
4. Inspected the Git history for the axis-3 reference commit (`cdff6f09d`) which
   added `+ std::marker::Sync` to `wrap_governance_inbound`.

## Inventory

### `#[async_trait]` usages found (7 total)

| File | Lines | Purpose |
|---|---|---|
| `inbox.rs:443` | `GovernanceInboundActivity` trait def | Inbound activity discriminator trait |
| `publish_label.rs:7` | `impl Activity for PublishLabel` | AP Activity impl |
| `publish_label.rs:35` | `impl GovernanceInboundActivity for PublishLabel` | Inbound trait impl |
| `publish_sanction_notice.rs:46` | `impl Activity for PublishSanctionNotice` | AP Activity impl |
| `publish_sanction_notice.rs:110` | `impl GovernanceInboundActivity for PublishSanctionNotice` | Inbound trait impl |
| `publish_trust_attestation.rs:32` | `impl Activity for PublishTrustAttestation` | AP Activity impl |
| `publish_trust_attestation.rs:97` | `impl GovernanceInboundActivity for PublishTrustAttestation` | Inbound trait impl |

None in `crates/api/api/src/governance/` or `crates/db_schema/src/source/governance/`.

### Generic functions with type parameters found

| File | Fn | Generic | Notes |
|---|---|---|---|
| `inbox.rs:485` | `wrap_governance_inbound<'a, F, Fut, A>` | `A: GovernanceInboundActivity + Sync + 'a` | The central audit target |
| `publish_sanction_notice.rs:522` | `generate_governance_activity_id<T>` | `T: ToString` | Simple utility; no async |
| `publish_trust_attestation.rs:327` | `generate_governance_activity_id<T>` | `T: ToString` | Sibling of above; no async |

### `GovernanceInboundActivity` trait definition (`inbox.rs:443-462`)

```rust
#[async_trait::async_trait]
pub(crate) trait GovernanceInboundActivity: Sized {
  fn activity_id(&self) -> &Url;
  fn actor_domain(&self) -> LemmyResult<String>;
  fn payload_size_bytes(&self) -> LemmyResult<usize>;
  fn payload_size_cap_key(&self) -> &'static str;
  async fn check_per_actor_rate_limit(
    &self,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> { ... default no-op ... }
}
```

Key observations:
- Supertrait: `Sized` only (no explicit `Send + Sync`).
- Macro: `#[async_trait::async_trait]` WITHOUT `?Send` — the generated code for
  `check_per_actor_rate_limit` returns `Pin<Box<dyn Future + Send + 'async_trait>>` and
  requires `where Self: Send + Sync + 'async_trait`. Every implementor is therefore
  implicitly required to be `Send + Sync` by the macro-generated bounds.

### `wrap_governance_inbound` current bounds (`inbox.rs:490-493`)

```rust
where
  F: FnOnce(A, &'a Data<LemmyContext>) -> Fut,
  Fut: std::future::Future<Output = LemmyResult<()>> + 'a,
  A: GovernanceInboundActivity + std::marker::Sync + 'a,
```

### Canonical sibling (`crates/apub/activities/src/lib.rs:122`)

```rust
where
  A: Activity + Serialize + Send + Sync + Clone + Activity<Error = LemmyError>,
  ActorT: Actor + GetActorType,
```

The `send_lemmy_activity` function explicitly enumerates `Send + Sync` on its `A: Activity`
type parameter. The Activity trait itself (from `activitypub_federation`) also uses
`#[async_trait]` without `?Send`, making Send+Sync implicit — but the sibling names them
explicitly.

## Findings

**Tier 1:** 0
**Tier 2:** 1
**Tier 3:** 1

---

### Finding F3-1 (Tier 2) — `wrap_governance_inbound` missing explicit `+ Send` on `A`

**File:** `crates/apub/activities/src/governance/inbox.rs:493`
**Current:** `A: GovernanceInboundActivity + std::marker::Sync + 'a`
**Convention sibling:** `lib.rs:122` → `A: Activity + Serialize + Send + Sync + Clone + ...`

The current bound names `Sync` explicitly but omits `Send`. The sibling
`send_lemmy_activity` (the only other generic async activity function in the same crate)
names both `Send + Sync` explicitly on its activity type parameter.

**Why not Tier-1:** The `GovernanceInboundActivity` trait is decorated with
`#[async_trait::async_trait]` without `?Send`, which means the macro's generated code
requires `Self: Send + Sync + 'async_trait` on every implementor (enforced at impl sites
`publish_label.rs:35`, `publish_sanction_notice.rs:110`,
`publish_trust_attestation.rs:97`). Additionally, the `activitypub_federation` framework's
`receive_activity` and `receive_activity_with_hook` require `A: Activity + Send + 'static`
(confirmed at `activitypub_federation-0.7.0-beta.10` `src/actix_web/inbox.rs:24`), so all
concrete callers of `wrap_governance_inbound` are already forced to be `Send` by the
framework. No weakened enforcement exists in practice — all three concrete callers
(`PublishLabel`, `PublishSanctionNotice`, `PublishTrustAttestation`) are `Send` by construction.

**Classification:** Tier-2 (sibling divergence in explicit naming convention; no weakened
runtime enforcement). Future `GovernanceInboundActivity` implementors intended for use
OUTSIDE the AP framework dispatch path (e.g., test mocks) could be `!Send` and still satisfy
the wrapper's current bounds, since the explicit `+ Send` is absent.

**Evidence string:**
`axis-3: inbox.rs:493 new=<A:GovernanceInboundActivity+Sync+'a> sibling=lib.rs:122 sibling=<A:Activity+Send+Sync+...>`

---

### Finding F3-2 (Tier 3) — `GovernanceInboundActivity` trait lacks explicit `: Send + Sync` supertraits

**File:** `crates/apub/activities/src/governance/inbox.rs:444`
**Current:** `pub(crate) trait GovernanceInboundActivity: Sized`
**Convention sibling (reference):** `activitypub_federation Activity` pattern — traits
intended for async dispatch explicitly document their thread-safety requirements in the
supertrait list rather than relying on `async_trait` macro side-effects.

The trait definition expresses only `Sized` as a supertrait. The `Send + Sync` constraints
are real (enforced by the `#[async_trait::async_trait]` expansion) but are invisible from
the trait declaration. A reader of the trait signature cannot determine thread-safety
requirements without knowing the `async_trait` macro's transformation rules.

**Why Tier-3 (not Tier-2):** There is no direct in-file sibling with explicit `: Send + Sync`
on a custom governance trait to compare against; the closest sibling (the `Activity` trait)
is from an external crate and uses the same `#[async_trait]` pattern. The finding is
stylistic — documentation/readability rather than divergence from an established in-codebase
contract.

**Evidence string:**
`axis-3: inbox.rs:444 trait=GovernanceInboundActivity:Sized (missing explicit :Send+Sync) convention=async_trait_without_?Send_implicitly_enforces_Send+Sync`

## Historical context

The spec's reference commit `cdff6f09d` ("finalize fix-impl-1 — inbox helpers &mut DbConn +
Sync bound") added `+ std::marker::Sync` to `wrap_governance_inbound` (commit
`b9691c0ab`). The pre-fix state was `A: GovernanceInboundActivity` (missing Sync entirely),
which caused an `E0277` compile error at two call sites. That finding (the one the spec uses
as its example) is **fully resolved** on trunk. A subsequent fix (`3bd2cfa4`, fix-impl-4
HRTB) added the `'a` lifetime parameter. Both fixes are present on `governance-v0 @
7e6c4202f`.

## Hypothesis discipline

Both findings were tested against the following criteria:

1. **Tier-1 gate:** Does trunk cargo check exit 0? Yes (confirmed in brief). Could a
   concrete `!Sync` or `!Send` type instantiate `wrap_governance_inbound`? No — async_trait
   without `?Send` generates `Self: Send + Sync` bounds on the method's return-type future.
   Any `!Send` or `!Sync` type fails at the `impl GovernanceInboundActivity` site, not at
   the wrapper's call site. No weakened enforcement path exists. → NOT Tier-1.

2. **Tier-2 gate:** Is there a sibling with strictly stricter explicit bounds? Yes — `lib.rs:122`
   has `+ Send + Sync` both explicit on an analogous activity generic function. The wrapper
   names only `+ Sync`. This is a real divergence in convention, not in runtime behaviour.
   F3-1 is Tier-2.

3. **Tier-3 gate:** Is the remaining finding (F3-2) a documentation/style gap without
   sibling precedent? Yes. F3-2 is Tier-3.

## Evidence strings

- F3-1: `axis-3: inbox.rs:493 new=<A:GovernanceInboundActivity+Sync+'a> sibling=lib.rs:122 sibling=<A:Activity+Send+Sync+...>`
- F3-2: `axis-3: inbox.rs:444 trait=GovernanceInboundActivity:Sized (missing :Send+Sync explicit) implicit_via_async_trait`
