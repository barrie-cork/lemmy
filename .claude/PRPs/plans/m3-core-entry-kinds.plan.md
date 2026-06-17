# Plan: M3-core entry kinds (Phase 2)

## Summary
Register **3 new** chair/mute governance-log entry-kind string consts for M3 town halls — `ENTRY_KIND_ROOM_CHAIR_TRANSFERRED`, `ENTRY_KIND_ROOM_CHAIR_OVERRIDE`, `ENTRY_KIND_ROOM_MUTE_ALL` — mirroring the M2 room-kinds pattern exactly. Zero-migration (`entry_kind` is `TEXT`). The Junior touches exactly **2 `crates/` files** (the canonical const block in `db_schema`, plus the `api` shim re-export AND its `ROOM_KINDS` validation array); the advisor reconciles the `.claude/` registry doc separately on `governance-v0`. No handlers, no call sites (those land in M3 phase 3/4 bridge-side), no schema change. Total entry-kind count 69 → 72.

## Source
- [m3-town-halls-rtc.prd.md](.claude/PRPs/prds/m3-town-halls-rtc.prd.md) §Implementation Phases — Phase 2 "M3-core entry kinds"; §Technical Approach (crates affected); §Decisions Log D3 (3 new consts + emit existing recording kind)
- Relevant [04](docs/brehon-law-inspired-network/04-data-model-and-api.md) sections: §11 "Entry-kind registry" (line 470) — **stale, advisor doc-drift, out of Junior scope**
- Relevant ADRs from [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md): **ADR-008** (append-only signed governance log; `entry_kind` TEXT, new consts only, zero-migration)

## Problem Statement
M3 town halls emit three chair/mute lifecycle records onto the governance hash chain — a chair delegating the floor (`room_chair_transferred`), a chair force-demoting/promoting a participant (`room_chair_override`), and a federation-wide emergency mute (`room_mute_all`). These string consts must exist (and be reachable via the `api` shim + accepted by the `append_room_event` gate) before the M3 phase-3/phase-4 bridge call sites can reference them. This phase registers the consts only; it adds no emitters.

## Solution Statement
Append 3 consts to the M2 room-kinds block in `crates/db_schema/src/source/governance/governance_log.rs`, add them (alphabetically) to the `pub use` shim in `crates/api/api/src/governance/governance_log.rs`, and extend that file's `ROOM_KINDS` runtime-validation array (the ADR-008 integrity gate for `append_room_event`) from 10 → 13 entries. The advisor separately adds the registry-doc section and bumps the count invariant 69 → 72 on `governance-v0` (advisor-owned `.claude/**`).

## Metadata

| Field | Value |
|---|---|
| Type | SCHEMA (registry const, no migration) |
| Complexity | LOW |
| Crates Affected | `lemmy_db_schema`, `lemmy_api` |
| v0 Step | n/a — M-track (post-v1) |
| Dependencies | M2 (shipped: PRs #191/#196/#197); the M2 room-kinds block + `append_room_event`/`ROOM_KINDS` gate already on `governance-v0` |
| Estimated Tasks | 4 (2 Junior `crates/` edits → 1 validation → 1 advisor `.claude/` reconcile) |

---

## Flow Design

### Before State
```
crates/db_schema/.../governance_log.rs
  // M2 room kinds (10) — 10 ENTRY_KIND_ROOM_* consts   ← RECORDING_UPLOADED present (line 243)
crates/api/.../governance_log.rs
  pub use { ...69 consts... }                            ← no chair/mute kinds
  const ROOM_KINDS: &[&str] = &[ ...10 room kinds... ]   ← append_room_event rejects chair/mute
.claude/rules/governance-log-entry-kind-registry.md
  ## ...m2-late-b-actor (2)...                           ← no M3 section; invariant count = 69
```

### After State
```
crates/db_schema/.../governance_log.rs
  // M2 room kinds (10) ...
  // M3 town-hall chair/mute kinds (3) — zero-migration; TEXT
  ENTRY_KIND_ROOM_CHAIR_TRANSFERRED / _CHAIR_OVERRIDE / _MUTE_ALL   ← 3 new consts
crates/api/.../governance_log.rs
  pub use { ...72 consts (3 new, alphabetical) ... }
  const ROOM_KINDS: &[&str] = &[ ...13 room kinds... ]              ← gate accepts all 3
.claude/rules/governance-log-entry-kind-registry.md
  ## M3 town-hall chair/mute kinds (3, m3-core-entry-kinds)         ← new section
  ## Acceptance invariants ... 72 ...                               ← count bumped
```

### Endpoint Changes
None. No route, no handler, no DTO. This phase only registers consts + widens the `append_room_event` accept-set. (Call sites = M3 phase 3/4, bridge-side.)

---

## Mandatory Reading (implementation agent MUST read before starting)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `crates/db_schema/src/source/governance/governance_log.rs` | 236-260 | The M2 room-kinds block to MIRROR + the m2-late sections immediately after (insertion point + exact const syntax) |
| P0 | `crates/api/api/src/governance/governance_log.rs` | 39-72, 97-132 | The `pub use` shim (alphabetical insertion) + the `ROOM_KINDS` array + `append_room_event` gate doc comments to bump 10→13 |
| P1 | [m3-town-halls-rtc.prd.md](.claude/PRPs/prds/m3-town-halls-rtc.prd.md) | §Decisions Log D3, §Proposed Solution item 3 | The 3 const names + their `&str` values + the "emit existing recording kind" note (no new const for recording) |
| P1 | [99 ADR-008](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | ADR-008 | `entry_kind` is append-only TEXT; new consts only, no migration, no chain rewrite |

**External Documentation:** None — this is a pure pattern-mirror within the existing workspace.

---

## Patterns to Mirror

**ENTRY_KIND_CONST_BLOCK (canonical definitions):**
```rust
// SOURCE: crates/db_schema/src/source/governance/governance_log.rs:236-248
// COPY THIS PATTERN (section comment + contiguous consts, type-annotated, snake_case value):
// M2 room kinds (10) — zero-migration; entry_kind is TEXT
// Call sites land in the bridge-side plan (pending) per the pre-landed-const
// exemption. The append_room_event wrapper (Task 5) provides the typed gated path.
pub const ENTRY_KIND_ROOM_CREATED: &str = "room_created";
pub const ENTRY_KIND_ROOM_ARCHIVED: &str = "room_archived";
// ... (8 more) ...
pub const ENTRY_KIND_ROOM_LIFECYCLE_EVENT: &str = "room_lifecycle_event";
```

**SHIM_RE_EXPORT (alphabetical `pub use`):**
```rust
// SOURCE: crates/api/api/src/governance/governance_log.rs:60-63
// COPY THIS PATTERN — consts are alphabetical within the single pub use block:
  ENTRY_KIND_ROLLUP_RECOMPUTED, ENTRY_KIND_ROOM_ARCHIVED, ENTRY_KIND_ROOM_BRIDGE_ERROR,
  ENTRY_KIND_ROOM_CREATED, ENTRY_KIND_ROOM_DECISION_RELAYED, ENTRY_KIND_ROOM_IDENTITY_REVEALED,
  ENTRY_KIND_ROOM_LIFECYCLE_EVENT, ENTRY_KIND_ROOM_MEMBER_ADDED, ENTRY_KIND_ROOM_MEMBER_REMOVED,
  ENTRY_KIND_ROOM_RECORDING_UPLOADED, ENTRY_KIND_ROOM_TRANSCRIPT_READY,
```

**ROOM_KINDS_VALIDATION_ARRAY (ADR-008 integrity gate):**
```rust
// SOURCE: crates/api/api/src/governance/governance_log.rs:97-111
// COPY THIS PATTERN — sorted array; doc comment count must match len; bump 10→13:
/// The 10 allowed `entry_kind` values for [`append_room_event`].
/// Any kind not in this set is rejected (ADR-008 integrity gate).
#[cfg(feature = "full")]
const ROOM_KINDS: &[&str] = &[
  ENTRY_KIND_ROOM_ARCHIVED,
  ENTRY_KIND_ROOM_BRIDGE_ERROR,
  // ... (8 more, alphabetical) ...
  ENTRY_KIND_ROOM_TRANSCRIPT_READY,
];
```

**REGISTRY_DOC_SECTION (advisor-authored, mirror m2-late-b-actor):**
```markdown
<!-- SOURCE: .claude/rules/governance-log-entry-kind-registry.md:270-288 -->
## m2-late-b-actor entry kinds (2, this sub-phase)
... prose ...
| Rust const | `&str` value | Source | Emitting handler | Semantic |
|---|---|---|---|---|
| `ENTRY_KIND_ACTOR_APP_LINK_CREATED` | `actor_app_link_created` | ... | ... | ... |
```

---

## Files to Change

| File | Action | Owner | Justification |
|---|---|---|---|
| `crates/db_schema/src/source/governance/governance_log.rs` | UPDATE | Junior | Add 3 `ENTRY_KIND_ROOM_*` consts after the M2 block |
| `crates/api/api/src/governance/governance_log.rs` | UPDATE | Junior | Add 3 consts to `pub use` shim (alphabetical) + extend `ROOM_KINDS` 10→13 + bump 2 doc-comment counts |
| `.claude/rules/governance-log-entry-kind-registry.md` | UPDATE | **Advisor** | New M3 section + bump invariant count 69→72 (advisor-owned `.claude/**`) |

**Out of scope (deliberately NOT in this plan):**
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` §11 says "55 valid kinds" (lines 131, 474) — already stale at M2 (should read 69). This is a **pre-existing advisor doc-drift** independent of M3; fix it in a separate `docs(drift):` pass, not as part of the Junior's 2-file edit. Noted here so it isn't mistaken for an M3 regression.

---

## NOT Building (v0/M3 scope limits)

- **No new const for recording** — `ENTRY_KIND_ROOM_RECORDING_UPLOADED` is already registered (M2, `governance_log.rs:243`). M3 *emits* it in phase 5; this phase adds nothing for it. (PRD D3.)
- **No emitters / call sites** — the bridge writes these kinds via `append_room_event` in M3 phase 3 (`chair_transferred`/`chair_override`) and phase 4 (`mute_all`). Pre-landed-const exemption applies, exactly as the M2 room block landed before its bridge call sites.
- **No migration** — `entry_kind` is `TEXT` (ADR-008); registering a const is a code-only change.
- **No payload structs** — `RoomEventPayload` already exists from M2; M3 phases 3/4 extend payloads bridge-side if needed, not here.

---

## Step-by-Step Tasks

Execute in order. One commit per task.

### Task 1: ADD 3 consts to `crates/db_schema/src/source/governance/governance_log.rs`
- **ACTION**: After the M2 room-kinds block (after `ENTRY_KIND_ROOM_LIFECYCLE_EVENT`, ~line 248, before the `// m2-late-1 additions` comment at ~line 250), insert a new section comment + 3 consts.
- **IMPLEMENT** (exact text):
  ```rust
  // M3 town-hall chair/mute kinds (3) — zero-migration; entry_kind is TEXT.
  // Call sites land bridge-side in M3 phase 3 (_CHAIR_TRANSFERRED / _CHAIR_OVERRIDE)
  // and phase 4 (_MUTE_ALL) via append_room_event. Pre-landed-const exemption.
  pub const ENTRY_KIND_ROOM_CHAIR_TRANSFERRED: &str = "room_chair_transferred";
  pub const ENTRY_KIND_ROOM_CHAIR_OVERRIDE: &str = "room_chair_override";
  pub const ENTRY_KIND_ROOM_MUTE_ALL: &str = "room_mute_all";
  ```
- **MIRROR**: `crates/db_schema/src/source/governance/governance_log.rs:236-248` (the M2 block — same const syntax, same section-comment style).
- **GOTCHA**: Do NOT alphabetize within the file's const list — the existing sections are grouped by milestone in semantic order, not alphabetical. Append as a new dated section.
- **GOTCHA**: `&str` values are snake_case and must exactly match the const-name suffix lowercased (`ENTRY_KIND_ROOM_MUTE_ALL` → `"room_mute_all"`). A literal that collides with any existing value fails the acceptance invariant.
- **VALIDATE**: `cargo check -p lemmy_db_schema`

### Task 2: ADD 3 consts to the shim + extend `ROOM_KINDS` in `crates/api/api/src/governance/governance_log.rs`
- **ACTION (part a — `pub use` shim, alphabetical)**: Insert the 3 consts into the `pub use lemmy_db_schema::...::{ ... }` block (lines 39-72) at their alphabetical positions:
  - `ENTRY_KIND_ROOM_CHAIR_OVERRIDE` and `ENTRY_KIND_ROOM_CHAIR_TRANSFERRED` go **after** `ENTRY_KIND_ROOM_BRIDGE_ERROR`, **before** `ENTRY_KIND_ROOM_CREATED` (CHAIR_OVERRIDE before CHAIR_TRANSFERRED — `O` < `T`).
  - `ENTRY_KIND_ROOM_MUTE_ALL` goes **after** `ENTRY_KIND_ROOM_MEMBER_REMOVED`, **before** `ENTRY_KIND_ROOM_RECORDING_UPLOADED`.
- **ACTION (part b — `ROOM_KINDS` array)**: Add all 3 consts to the `ROOM_KINDS` array (lines 100-111) in sorted order matching the shim (CHAIR_OVERRIDE, CHAIR_TRANSFERRED after BRIDGE_ERROR; MUTE_ALL after MEMBER_REMOVED). Bump the doc comment on line 97 from "The 10 allowed" → "The 13 allowed" and line ~115 from "the 10 `ENTRY_KIND_ROOM_*` kinds" → "the 13 `ENTRY_KIND_ROOM_*` kinds".
- **MIRROR**: `crates/api/api/src/governance/governance_log.rs:60-63` (shim) + `:97-111` (`ROOM_KINDS`).
- **GOTCHA**: This is a **2-part edit in one file** — both the `pub use` block AND the `ROOM_KINDS` array. Missing `ROOM_KINDS` would compile fine but leave `append_room_event` rejecting the new kinds at runtime — the M3 phase-3/4 emitters would fail. Both parts are required.
- **GOTCHA**: `ROOM_KINDS` is `#[cfg(feature = "full")]` — validation only matters under `--features full`; `cargo check -p lemmy_api` (no features) won't catch a stale doc-comment count. The count parity is enforced by the acceptance invariant in Task 4, not the compiler.
- **VALIDATE**: `cargo check -p lemmy_api --features full`

### Task 3: VALIDATE workspace + collision/parity invariants (laptop)
- **ACTION**: Write a `validate-pending-laptop` DQ entry with `commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`, commit + push, then **stop**. Do NOT run cargo yourself (NO-CARGO-ON-ELITEDESK). The laptop advisor runs it.
- **POST-VALIDATION INVARIANT CHECKS** (advisor runs after cargo passes — these are the PRD's "collision check (A == B, no dup literals)" + shim parity):
  ```bash
  # A: const count in db_schema — expect 72
  rg '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs | wc -l
  # B: no duplicate string-literal values — expect empty
  rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs \
    | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort | uniq -d
  # Shim parity: api re-export count == db_schema define count (72)
  rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l
  ```
- **EXPECT**: A = 72, B = empty, shim count = 72.
- **VALIDATE**: `cargo-check.sh --workspace --features full` exit 0 + the 3 greps above.

### Task 4: ADVISOR — reconcile registry doc on `governance-v0`
- **ACTION** (advisor-owned `.claude/**`, NOT a Junior task — author on `governance-v0`): Add an M3 section to `.claude/rules/governance-log-entry-kind-registry.md` after the m2-late-b-actor section (~line 288) and bump the count invariant.
- **IMPLEMENT (new section, mirror m2-late-b-actor format)**:
  ```markdown
  ## M3 town-hall chair/mute kinds (3, m3-core-entry-kinds)

  Landed in M3 phase-2's two-file edit (`db_schema` consts + `api` shim/`ROOM_KINDS`).
  Call sites land bridge-side: `_CHAIR_TRANSFERRED` + `_CHAIR_OVERRIDE` in M3 phase 3,
  `_MUTE_ALL` in M3 phase 4 — all via `append_room_event`. Pre-landed-const exemption applies.

  | Rust const | `&str` value | Source | Emitting handler | Semantic |
  |---|---|---|---|---|
  | `ENTRY_KIND_ROOM_CHAIR_TRANSFERRED` | `room_chair_transferred` | M3 phase-2 const; phase-3 bridge call site (pending) | bridge stage-mode controller (pending) | Chair delegates the floor mid-session via Matrix-native transfer. Payload: `{from_pseudonym, to_pseudonym, at}` (pseudonyms survive scrub; ADR-015) |
  | `ENTRY_KIND_ROOM_CHAIR_OVERRIDE` | `room_chair_override` | M3 phase-2 const; phase-3 bridge call site (pending) | bridge stage-mode controller (pending) | Chair force-demotes/promotes a participant. Payload: `{action, target_pseudonym}` |
  | `ENTRY_KIND_ROOM_MUTE_ALL` | `room_mute_all` | M3 phase-2 const; phase-4 bridge call site (pending) | bridge emergency-mute path (pending) | Federation-wide emergency mute; all non-chair publishers dropped. Payload: `{chair_pseudonym, federated: true}` |
  ```
- **IMPLEMENT (count bump)**: In `## Acceptance invariants` (~line 291), change **`69`** → **`72`** and extend the breakdown by appending `+ 3 m3-core-entry-kinds`.
- **VALIDATE**: re-run the Task-3 greps; registry section present (`rg 'M3 town-hall' .claude/rules/governance-log-entry-kind-registry.md`); invariant reads 72.

---

## Testing Strategy

Per [m3-town-halls-rtc.prd.md](.claude/PRPs/prds/m3-town-halls-rtc.prd.md) §Success Criteria, the entry-kind row is verified by `cargo check --workspace` + registry-doc grep + collision check — **not** by a new e2e test. The M2 room kinds shipped the same way (no per-const test). The emitter behaviour is tested in M3 phases 3-5 (bridge-side integration tests), not here.

### Tests to Add
None. (Const registration is compile-time + grep-verified, consistent with all prior entry-kind sub-phases.)

### Edge Cases
- [ ] No string-literal collision with any of the 69 existing values (Task-3 invariant B = empty)
- [ ] Shim re-export count == db_schema define count == 72 (Task-3 parity grep)
- [ ] `ROOM_KINDS` len == 13 and doc comment says "13" (so M3 phase-3/4 `append_room_event` accepts the new kinds)
- [ ] `cargo check --workspace --features full` is clean (the `#[cfg(feature = "full")]` `ROOM_KINDS` path compiles)

---

## Validation Commands

This is a Rust project — no npm/pnpm. Cargo runs on the **laptop** (NO-CARGO-ON-ELITEDESK); the Junior writes a `validate-pending-laptop` DQ and stops.

### Level 1: STATIC_ANALYSIS
```bash
cargo check --workspace --features full
cargo clippy --workspace --features full -- -D warnings
```
**EXPECT**: Exit 0, zero errors, zero warnings.

### Level 2: INTEGRATION_TESTS
Not applicable — no new test in this phase.

### Level 3: FULL_BUILD
```bash
cargo build --workspace --features full
```
**EXPECT**: Exit 0.

### Level 4: MIGRATION_VALIDATION
Not applicable — zero-migration (`entry_kind` is TEXT).

### Level 5: REGISTRY_INVARIANTS (the PRD's collision check)
```bash
rg '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs | wc -l   # == 72
rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs \
  | awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort | uniq -d                     # empty
rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l                     # == 72
rg 'M3 town-hall' .claude/rules/governance-log-entry-kind-registry.md                            # section present
```
**EXPECT**: 72 / empty / 72 / one match.

---

## Acceptance Criteria

- [ ] 3 new `ENTRY_KIND_ROOM_*` consts defined in `db_schema` with correct snake_case values
- [ ] All 3 re-exported via the `api` shim in alphabetical order
- [ ] `ROOM_KINDS` array extended to 13 entries; doc comments updated 10→13
- [ ] Level 1 + Level 3 pass exit 0 (`--features full`)
- [ ] Level 5 invariants pass (count 72, no collisions, shim parity, registry section present, count bumped 69→72)
- [ ] No new `cargo clippy --features full` warnings
- [ ] No contradiction with ADR-008 (zero-migration, append-only, new consts only)
- [ ] Registry doc M3 section authored by advisor on `governance-v0`

## Completion Checklist
- [ ] Task 1 (db_schema consts) committed + Level-1 clean
- [ ] Task 2 (shim + `ROOM_KINDS`) committed + Level-1 `--features full` clean
- [ ] Task 3 validate-pending-laptop DQ written, pushed; laptop ran cargo + invariants
- [ ] Task 4 advisor reconciled registry doc; count reads 72
- [ ] All Level-5 invariants green

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Junior edits `db_schema` consts but forgets `ROOM_KINDS` array | MED | MED | Task 2 is explicitly a 2-part edit with a GOTCHA; Level-5 has no direct `ROOM_KINDS`-len grep, so the brief §4 must call out "len == 13" — advisor spot-checks the diff |
| String-literal collision with an existing value | LOW | HIGH | Snake_case values are unique by construction (chair/mute have no prior kind); Task-3 invariant B catches it |
| Shim alphabetical position wrong (compiles, but drifts from convention) | LOW | LOW | Exact positions named in Task 2; advisor diff-checks against the MIRROR snippet |
| `04` doc-drift mistaken for M3 regression | LOW | LOW | Flagged explicitly in "Files to Change" out-of-scope note; handled in a separate `docs(drift):` pass |

---

## Notes
- This is the M2 room-kinds pattern applied a third time (after m2-core-hook's 10 and m2-late's 2+2). The only twist vs. the m2-late-b-actor precedent: M2's room block predates the `ROOM_KINDS` gate's existence, so the gate already lists 10 — M3 is the **first** chair/mute sub-phase that must remember to extend `ROOM_KINDS`. That's the one non-mechanical edge and is called out in Task 2.
- `ENTRY_KIND_ROOM_RECORDING_UPLOADED` (line 243) is untouched here — it's already registered and already in `ROOM_KINDS`; M3 phase 5 emits it.
