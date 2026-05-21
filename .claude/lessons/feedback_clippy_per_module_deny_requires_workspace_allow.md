---
name: Clippy per-module #![deny] requires workspace-allow when lint is in a default-deny group
description: When a plan prescribes per-module #![deny(<lint>)] for a Clippy lint that lives inside a workspace-default-active group (e.g. style, perf, suspicious), per-module deny is a no-op — the lint is ALREADY firing workspace-wide via the group's level. The plan must (a) audit the workspace-level [workspace.lints.clippy] group levels, (b) add an explicit <lint> = "allow" override at workspace level, then (c) re-enable via per-module #![deny] on the target modules. Rustc lint-precedence rule 4 (lower-syntax-tree wins) makes the inversion stable. Surfaced at brehon-conformance-audit DQ #311 cycle-3 catch-fire 2026-05-21 after ~123 min wallclock on three failed fix-impl cycles. See [[project_phase6_convention_divergence_class]] for the broader defect class.
type: feedback
originSessionId: brehon-conformance-audit-retro
---

## What this catches

The failure-mode where a planner prescribes per-module `#![deny(clippy::<lint>)]` as the enforcement mechanism, but the lint lives in a workspace-default-active group (e.g. `style`, `perf`, `suspicious`, `pedantic` when enabled). In that case the lint is **already firing workspace-wide** via the group level set in `Cargo.toml [workspace.lints.clippy]`. Per-module `#![deny]` is a no-op; the per-module annotation makes ZERO difference. What the plan ACTUALLY needs is the **inverse**: workspace-level override to **allow** the lint, then per-module `#![deny]` to re-enable only where conformance matters.

The rule that makes this safe: **rustc lint-precedence rule 4 — lower-syntax-tree wins**. A workspace-level `<lint> = "allow"` is at the highest scope (workspace). A per-module `#![deny(<lint>)]` is at a lower scope (module root). The lower scope wins. Workspace-allow silences the lint workspace-wide; per-module deny re-enables on the named modules only.

## Why this fires

The plan author thinks of `disallowed_methods` (or any specific lint name) as a "lint we're enabling" — independent of its group membership. Clippy's group membership is the wrinkle:

- `clippy::disallowed_methods` is in the `style` group.
- Many Brehon-fork repos (including this one) set `style = { level = "deny", priority = -1 }` in `[workspace.lints.clippy]` at line ~100 of root `Cargo.toml`.
- Adding `disallowed-methods = [...]` entries to `clippy.toml` activates them globally because the group is already `deny`.
- Per-module `#![deny(clippy::disallowed_methods)]` is then redundant — the lint was ALREADY firing on every module.

Net effect: workspace-wide enforcement on legacy/shared crates triggers cascading fix-impl cycles. Each cycle's narrow patch lands the cited callsites; the next cycle's workspace check fails on previously-unflagged crates. The pattern reproduces until either (a) every callsite is patched (high cost on shared crates with legitimate uses of the disallowed method), or (b) advisor cycle-count meta-rule fires HARD REFUSAL after 3 cycles.

## The recipe

When the plan prescribes a per-module enforcement gate for a specific Clippy lint:

1. **Identify the lint's group.** Use `cargo clippy --workspace -- -W clippy::<lint> --explain <lint>` or read `clippy::<lint>` documentation. Note the group memberships (a lint can be in multiple groups).

2. **Audit the workspace-level group levels.** Read `Cargo.toml [workspace.lints.clippy]` for every group the lint belongs to. Common Brehon-fork shape:
   ```toml
   [workspace.lints.clippy]
   style = { level = "deny", priority = -1 }
   ```

3. **Determine the workspace-wide default level for the lint.** If ANY containing group is `deny` AND no explicit `<lint> = "allow"` override exists, the workspace-wide default is `deny`.

4. **Decide direction:**
   - **Workspace default is the desired baseline?** Per-module `#![deny]` is redundant (no-op). Specify per-module `#![warn]` or `#![allow]` to RELAX, not strengthen, if applicable. Or drop the per-module attribute entirely.
   - **Workspace default is OPPOSITE the desired baseline?** You need both: workspace-level override + per-module re-enable. Specify in plan:
     ```toml
     # In root Cargo.toml [workspace.lints.clippy], add:
     disallowed_methods = "allow"  # Federation-only enforcement: per-module #![deny] re-enables.
     ```
     Then in the target module:
     ```rust
     #![deny(clippy::disallowed_methods)]
     ```

5. **Cite rustc lint-precedence rule 4 in the plan body.** Anchors the mechanism so a future reviewer can verify the direction is intentional.

## Worked example (brehon-conformance-audit retro)

**Plan original (broken):** `clippy.toml` adds `disallowed-methods` entries; per-module `#![deny(clippy::disallowed_methods)]` on three federation `mod.rs`. No mention of workspace lint-group level.

**Actual workspace state:** `Cargo.toml:100` says `style = { level = "deny", priority = -1 }`. `disallowed_methods` is in `style`. Workspace-wide default for `disallowed_methods` was ALREADY `deny`.

**What happened:** adding entries to `clippy.toml` activated workspace-wide enforcement on every crate. Per-module `#![deny]` was a no-op. Three fix-impl cycles burned trying to patch shared-crate violations:
- fix-impl-1: lemmy_utils 6 sites
- fix-impl-3: lemmy_diesel_utils 4 sites
- Narrow-probe gate: still failed on lemmy_apub_objects pre-existing violations

**Catch-fire at cycle-3.** User option-c mechanism revision (DQ #311) inverted the plan:

```toml
# Cargo.toml [workspace.lints.clippy]
disallowed_methods = "allow"  # Federation-only enforcement via per-module deny.
```

```rust
// crates/apub/activities/src/governance/mod.rs:1
// crates/api/api/src/governance/mod.rs:12 (after //! block)
// crates/db_schema/src/source/governance/mod.rs:1
#![deny(clippy::disallowed_methods)]
```

**Result:** workspace-wide `cargo clippy --workspace` is clean; federation governance modules enforce the gate; shared crates retain legitimate `.unwrap_or_default()` uses where empty-string-as-default is the correct semantic.

## When this applies

Any plan that prescribes a per-module `#![deny(clippy::<lint>)]` enforcement for a lint that lives inside `style`, `perf`, `suspicious`, `correctness`, or `complexity` group (these are clippy's default-warn or default-deny groups depending on repo config).

**Less commonly applies** to lints in `pedantic`, `restriction`, `nursery` — these groups are usually off at workspace level, so per-module `#![deny]` IS the activation. But verify the workspace config first.

## Cross-links

- [[project_phase6_convention_divergence_class]] — broader defect class this lesson defends against.
- [[feedback_clippy_test_style]] — workspace-default-deny `style` group pattern in Brehon-fork.
- [[feedback_fix_impl_pre_push_cargo_check.md]] — pre-push cargo-check gate that would catch this faster on the worker side.
- [[feedback_read_canonical_before_writing_spec.md]] — planner discipline for reading `Cargo.toml [workspace.lints.*]` before specifying lint gates.
- [[feedback_principles_not_rules.md]] — the lint-group-activation audit is a planner-side principle, not a hard rule.
