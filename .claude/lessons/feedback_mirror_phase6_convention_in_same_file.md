---
name: Mirror Phase-6 conventions in the same file
description: Phase-6 federation handlers + DB-source modules have established conventions per-file/per-module. New code added to the same file must mirror the canonical sibling (error idiom, conn type, append-reborrow, trait bounds, conn acquisition, ADR-015 pseudonym handling). The brehon-conformance-audit skill is the structural detection; Clippy disallowed_methods is the structural prevention. Surfaced at v1-federation-inbound-b fix-impl-1+2+3 (3 compile-caught + 1 latent footgun Finding 6.1).
type: feedback
originSessionId: a7c3f28e-8b14-4d6f-9e52-bca012345678
---

## What this catches

The **Phase-6 convention-divergence defect class**: new federation governance code adds a function to one of the three federation governance modules:

- `crates/apub/activities/src/governance/**.rs`
- `crates/api/api/src/governance/**.rs`
- `crates/db_schema/src/source/governance/**.rs`

...and the new function diverges from an established canonical-sibling pattern in the same file along one or more of six axes. Some divergences are compiler-caught (axis-1 conn-type, axis-3 trait-bound); others compile cleanly and surface as latent data-integrity or trust-boundary footguns at runtime (axis-4 error-idiom divergence). The existing tooling -- `security-auditor`, `code-audit`, `code-reviewer` -- does not catch this class; none perform a canonical-sibling-divergence pass.

## Evidence

Four incidents from `v1-federation-inbound-b`, documented in the Task 7 dogfood report at `.claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-2026-05-20.md`:

1. **fix-impl-1, incident 1 (axis-1):** Two helpers in `receive_remote_moderation_label` took `&mut AsyncPgConnection` as receiver but called `.run_transaction()` -- a method on `&mut DbConn<'_>`. Same-file siblings four lines away took `&mut DbConn<'_>`. Compiler-caught.

2. **fix-impl-1, incident 2 (axis-1):** Same root cause in a second helper in the same function. Compiler-caught.

3. **fix-impl-2, incident 3 (axis-3):** `wrap_governance_inbound<A: GovernanceInboundActivity>` was missing `+ Sync` on the generic bound. A same-file sibling had the complete bound. Compiler-caught.

4. **fix-impl-3, Finding 6.1 (axis-4 -- latent, compile-clean):** `receive_remote_moderation_label` at `crates/apub/activities/src/governance/inbox.rs:~743` used `.domain().map(str::to_string).unwrap_or_default()` to acquire `peer_domain`. This silently produces `source_instance = ""` for any remote actor without a domain. Two same-file siblings four lines away (`receive_remote_sanction_notice` L153, `receive_remote_trust_attestation` L265) use `.domain().ok_or_else(|| LemmyErrorType::Unknown(...))?.to_string()` -- a hard-error enforcement of the domain-presence contract. The `.unwrap_or_default()` variant **compiled cleanly under `cargo check --workspace --features full`** and was caught only because the user explicitly requested a thorough product-grade interpretation and the advisor ran a bespoke read-only subagent. Fixed at SHA `8b04e69a6`.

The dogfood report (`conformance-audit-v1-federation-inbound-b-dogfood-2026-05-20.md`) confirms the skill flags Finding 6.1 on Snapshot 1 (pre-fix) and clears it on Snapshot 2 (post-fix), achieving axis-4 precision = 1.000, recall = 1.000 on the calibration run.

## The six axes

Lifted verbatim from plan §10.1:

| Axis | What it means | Phase-6 instance | Detection method |
|---|---|---|---|
| **1. Conn-type / tx-boundary** | Does the new fn take `&mut DbConn<'_>` (starts tx via `.run_transaction`) or `&mut AsyncPgConnection` (must already be inside a tx)? | fix-impl-1 (#337): 2 helpers took `&mut AsyncPgConnection` but called `.run_transaction` (method on `&mut DbConn`); siblings 4 lines away took `&mut DbConn`. | Grep file for `.run_transaction(` callers; check receiver type vs sibling receiver type. LSP: `_hover` on receiver to confirm type. |
| **2. Append reborrow shape** | In-tx `governance_log::append` calls must reborrow `&mut (&mut *conn).into()` exactly. | Held (byte-conformant). | Pattern-grep against the canonical 4-token sequence. |
| **3. Trait-bound completeness** | `#[async_trait]` methods with `Sync`-requiring bodies need `A: Sync` on the generic. | fix-impl-1: `wrap_governance_inbound<A: GovernanceInboundActivity>` missing `+ Sync`; compile-caught. | Compiler is the oracle; planning-time MIRROR stub should compile-check (`cargo check --workspace --features full` reads existing runlog/DQ, never invoked by the skill). |
| **4. Error idiom at trust boundary** | `.domain().ok_or_else(|| LemmyErrorType::Unknown(...))?` (hard-error) vs `.unwrap_or_default()` (silent empty-string). | **Finding 6.1**: `receive_remote_moderation_label:~735` used `.unwrap_or_default()` -- persists `source_instance = ''`. Compiled cleanly. Latent data-integrity footgun. | **Sibling-diff**: locate same-file sibling doing same validation; flag every divergence where new code is *weaker* than sibling's enforced contract. Compiler-mechanical via Clippy `disallowed_methods` (Track B) at the trust-boundary modules. |
| **5. Conn acquisition idiom** | `let conn = &mut get_conn(pool).await?;` vs improvised variants. | Held (byte-conformant). | Pattern-grep `let \w+ = &mut get_conn(`. |
| **6. ADR-015 pseudonym handling for remote actors** | `None` for remote actors (no pseudonymisation possible). | Held. | Pattern-grep `actor_pseudonym` + remote-actor type. |

## The structural fix

Two layers:

1. **Skill (detection + prevention at brief-author time):** `.claude/skills/brehon-conformance-audit/` -- a read-only skill invoked in the advisor session at §3.1 (brief-author prevention checkpoint when the brief targets a federation module root) and §3.9 (retro detection checkpoint). The skill runs the six axes against an in-file sibling and emits a per-run audit report + per-run metrics file.

2. **Clippy gate (compiler-mechanical enforcement for axis-4):** `clippy.toml` seeds `disallowed-methods = ["Option::unwrap_or_default", "Result::unwrap_or_default"]` at workspace allow level. Three federation module roots each carry `#![deny(clippy::disallowed_methods)]`:
   - `crates/apub/activities/src/governance/mod.rs`
   - `crates/api/api/src/governance/mod.rs`
   - `crates/db_schema/src/source/governance/mod.rs`

   Mechanism: **rustc lint-precedence rule 4** -- workspace-level `allow` + lower-scope `deny` = the deny wins for code in the three federation `mod.rs` scopes only. Code outside those scopes sees only the workspace-level allow (no project-wide Clippy disruption). This enforces axis-4 trust-boundary discipline without a false-positive flood on the rest of the workspace.

## Brief-author checklist

If your impl-task brief creates or modifies a `fn` under one of the three federation governance module roots (`crates/apub/activities/src/governance/`, `crates/api/api/src/governance/`, `crates/db_schema/src/source/governance/`):

(a) **Read the same-file sibling first** -- `Read` the canonical sibling function with its full signature, receiver type, error idiom, and trait bounds before authoring the brief.

(b) **Cite the sibling in brief §3 Required reading** with the file path and line range, e.g. `crates/apub/activities/src/governance/inbox.rs:136-175 (receive_remote_sanction_notice -- axis-4 canonical error idiom)`.

(c) **Rerun the conformance-audit skill at brief time** with `target_scope = file <brief-named-file>` to surface any axis divergences before the impl-task worker writes the code.

## See also

- `[[feedback_plan_stub_uniformity_with_canonical_sibling]]` -- generalisation to any canonical-sibling shape miss (not just federation governance).
- `[[feedback_lemmy_error_no_std_error]]` -- axis-4 Case A/B/C enumeration; the recipe-family that .ok_or_else vs .unwrap_or_default is choosing between.
- `[[feedback_multi_write_handlers_need_transactions]]` -- axis-1 conn-type discipline source.
- `[[project_phase6_convention_divergence_class]]` -- user-directive PMD memory recording the six-axis checklist + latent-footgun evidence.
- `[[feedback_read_canonical_before_writing_spec]]` -- sibling-mirror discipline generalised to spec/template/rule authoring.
