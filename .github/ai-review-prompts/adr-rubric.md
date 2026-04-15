# ADR rubric — Brehon governance-v0

This rubric is the hard floor for every governance PR review. Cite ADR IDs
exactly as listed below. Do **not** hallucinate ADR numbers — only ADR-001
through ADR-015 exist.

## Hard constraints (ADRs — cannot be re-litigated in PR review)

- **ADR-001 — Fork of Lemmy.** v0 adds governance as new crates alongside
  upstream workspace members. Flag wholesale rewrites of upstream files where
  a new file or trait impl would suffice.

- **ADR-002 — No token-based governance.** Flag any diff that adds token,
  stake, wallet, ERC20, on-chain voting, or gas-fee references under
  `crates/api/api/src/governance/**` or `crates/db_schema/src/source/governance/**`.
  Influence derives from reputation dimensions only.

- **ADR-004 — Governance plane separated from content plane.** Governance
  handlers live under `crates/api/api/src/governance/**` and
  `crates/api/api_crud/src/governance/**`. Routes live under
  `/api/v4/governance/*`. Flag governance handlers merged into non-governance
  modules, and flag governance routes merged into non-governance routers.

- **ADR-006 — Inbound federation governance signals are advisory only.**
  Flag any inbound AP handler that mutates local case state, local reputation,
  or local sanctions on receipt of a federated `SanctionNoticeObject` or
  `TrustAttestationObject`. Storing for later human review is fine; auto-apply
  is not.

- **ADR-007 — MVP jury parameters: 5 jurors, quorum 3, simple majority.** No
  severity-based thresholds, no diversity constraints in v0. Flag diffs that
  try to implement v1 jury parameters inside v0.

- **ADR-008 — Append-only signed governance log.** Every write to case state,
  jury state, sanctions, appeals, and federation trust must emit a
  `governance_log` row **before** the user response returns. Flag handlers
  that return early without emitting. Flag migrations that DROP, ALTER,
  TRUNCATE, UPDATE, or DELETE from `governance_log`, or that change the
  hash-chain trigger function.

- **ADR-009 — Compatibility layer with existing Lemmy moderation.** A direct
  moderator action on a case under active jury review must move the case to
  `admin-review` state, not silently override the jury. Flag direct moderator
  actions that don't emit an admin-review transition.

- **ADR-010 — v0 scope = exactly the 11 endpoints from
  `05-mvp-and-delivery-plan.md` §2.** Flag new endpoints outside the allowlist
  (except the explicit `RevokeEndorsement` DTO exception in Phase 3 task 35,
  which is DTO-only — no endpoint). Flag any import, Cargo dependency, or
  config of Keycloak, OpenFGA, Vault, HSM, KMS, external log signers, or
  blockchain anchoring — those are v2 / v3.

- **ADR-011 — AGPLv3.** Flag any new source file added without an AGPLv3-
  compatible header, or any `Cargo.toml` dependency that is not AGPLv3-
  compatible (proprietary or GPL-incompatible-non-copyleft).

- **ADR-012 — Base is Lemmy 1.0-beta.** Use `ap_id`, not `actor_id`. Flag
  `actor_id` references added in new governance code — they're from the 0.19
  line and signal a stale copy-paste.

- **ADR-013 — `CaseStatus::EmergencyRemove` must exist and be handled
  exhaustively.** Flag any `match` on `CaseStatus` with a wildcard `_ =>` arm.
  Flag any diff that removes `CaseStatus::EmergencyRemove` or skips it in a
  pattern.

- **ADR-014 — Federation interop with vanilla Lemmy.** Content-level
  federation with vanilla Lemmy works. Governance AP activities are fork-only
  types; vanilla peers silently discard them. Flag diffs that assume vanilla
  peers can interpret `ModerationLabelObject`, `TrustAttestationObject`, or
  `SanctionNoticeObject`.

- **ADR-015 — GDPR: pseudonymised actor IDs in the governance log.** The log
  stores `actor_pseudonym` only, never usernames, emails, or display names.
  Every user-visible governance string must pass through `scrub()` before
  being written to any `public_case_log.rationale_redacted` or
  `governance_log.payload` field. Flag raw `person.name`, `person.email`,
  `display_name`, `.bio`, `.matrix_user_id` references near `governance_log`
  writes. Flag any `scrub_disabled`, `// skip scrub`, or
  `#[allow(...)].*scrub` bypass.

## Project rules

- **No DB mocking in integration tests.** Tests under `tests/e2e.rs` and
  `crates/*/tests/**` must hit a real Postgres. Flag `mock_db`, `MockDb`,
  `mockito.*postgres`, `InMemoryPgPool`, `FakeDbPool`, or equivalent.

- **Phase 2a Selectable template rule.** Governance view crates
  (`crates/db_views/governance_case`, `jury_queue`, `governance_modlog`,
  `reputation`) use bare-scalar fields that force the tuple-load + map
  pattern. **Never** `#[derive(Selectable)]` on these view structs — the
  Diesel macro tries to emit a SELECT expression for bare-scalar fields and
  fails with an unresolvable schema-module import error. Flag any PR that
  adds it.

- **Archived design docs are immutable.**
  `docs/brehon-law-inspired-network/chat1.md` and `chat2.md` are preserved as
  history. Flag any edit to either.

- **`main` branch is reserved for upstream rebases.** Governance feature work
  must not target `main`. Flag PRs opened against `main` that touch
  governance crates — those belong on `governance-v0`.

## What NOT to review

Silence is preferable to noise. Do not comment on:

- Upstream Lemmy conventions in non-governance paths (anything outside the
  "governance" namespace in each crate).
- Formatting, clippy, cargo build correctness, or cargo test outcomes —
  Woodpecker already runs these checks; duplicating them wastes reviewer
  bandwidth.
- Windows wrapper scripts under `scripts/brehon/*.bat` — Windows-dev-local
  only, not reachable from the Linux CI runners.
- Translation files, vendored lockfiles (`Cargo.lock`, `pnpm-lock.yaml`),
  auto-generated files.
- Style preferences (variable names, line length, comment phrasing) — only
  substantive ADR-class concerns.

## Output discipline

- Cite ADR IDs exactly (e.g. "ADR-008", not "ADR #8" or "the log ADR").
- Cite file paths with line numbers when quoting code (`file.rs:42`).
- If nothing is wrong, say "No ADR violations detected in this diff." and
  stop — do not pad with generic praise.
- Keep the whole review under 400 words. Terse is better than thorough.
