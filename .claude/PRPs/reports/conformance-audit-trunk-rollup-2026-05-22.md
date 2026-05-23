---
audit: brehon-conformance-audit (all 6 axes)
scope: trunk (governance-v0 @ 7e6c4202f)
date: 2026-05-22
mode: file (62 governance files in three roots)
cargo_gates: { check: pass (9m13s warm / 23m26s cold), clippy: pass (8m41s), test_no_run: in_progress }
---

# Brehon conformance audit — trunk roll-up

## Scope

All three federation/governance roots on `governance-v0` @ `7e6c4202f`:

- `crates/apub/activities/src/governance/` (5 files)
- `crates/api/api/src/governance/` (31 files)
- `crates/db_schema/src/source/governance/` (26 files)

Total: 62 governance .rs files. `phase-v1-federation-inbound-c` was merged
into `governance-v0` on 2026-05-21 (`7cfc21c23`) with `--delete-branch`, so
its merged code is part of this trunk audit; no separate fed-in-c re-audit
is needed (the local ref was removed at merge).

## Cargo gates

| Gate | Status | Wall clock | Notes |
|---|---|---|---|
| `cargo check --workspace --features full` (canonical, warm) | pass (exit 0) | 9m 13s | Trunk baseline before worktree was created. |
| `cargo check --workspace --features full` (worktree, cold) | pass (exit 0) | 23m 26s | Cold build, separate `target/`. |
| `cargo clippy --workspace --features full --no-deps -- -D warnings` | pass (exit 0) | 8m 41s | Zero clippy warnings under deny-warnings. |
| `cargo test --workspace --features full --no-run` | in flight | ~10–15 min expected | Will append exit on completion. |

The Phase-6 convention-divergence defect class is, by construction, the
*compile-clean* class: divergences that pass cargo gates but weaken
same-file siblings' enforced contracts (per
`feedback_verify_automated_reviewer_claims_against_compiler.md`). The
clean cargo result is therefore the floor, not the ceiling.

## Findings by axis

| Axis | Tier 1 | Tier 2 | Tier 3 | Files audited | Verdict |
|---|---:|---:|---:|---:|---|
| 1 conn-type / tx-boundary | 0 | 0 | 0 | 62 | Clean. 17 `.run_transaction(` callsites, all on `&mut DbConn<'_>`. 48 helper fns on `&mut AsyncPgConnection` are closure-body helpers (canonical). |
| 2 append reborrow | 0 | 20 | 0 | 62 | Sibling-divergence in token shape (3-token `&mut conn.into()` vs canonical 4-token `&mut (&mut *conn).into()`). Both produce identical `DbPool::Conn`. |
| 3 trait-bound completeness | 0 | 1 | 1 | 62 | Phase-6 axis-3 instance already fixed on trunk by `b9691c0ab`. Remaining gap: implicit-via-async-trait `Send` on `wrap_governance_inbound`. |
| 4 error idiom (trust boundary) | 0 | 0 | 5 | 62 | Phase-6 Finding 6.1 defect class **absent on trunk**. Zero `.unwrap_or_default()` hits. The 5 Tier-3 sites are URL composition + local JSONB projection. |
| 5 conn acquisition | 0 | 5 | 1 | 62 | 20 canonical conn-acquisition sites, 6 variants. Two clean unnecessary divergences (F1, F5); three with partial justification. |
| 6 ADR-015 pseudonym | 0 | 0 | 0 | 62 | ADR-015 / GDPR pseudonymisation contract uniformly held. Self-documenting via inline ADR-015 citations. |
| **Total** | **0** | **26** | **7** | **62** | **No catch-fire findings.** |

## Tier-1 findings (catch-fire candidates)

**None.** No enforced-contract weakening detected on `governance-v0` trunk
across any of the six axes. Trunk cargo check + clippy passed; the Phase-6
Finding 6.1 latent footgun pattern (`.unwrap_or_default()` at a federation
trust boundary with a sibling that hard-errors) is absent.

## Tier-2 findings (sibling-divergence; watch items for next sub-phase plan)

**26 Tier-2 findings across three axes.** Each is a sibling-divergence
without proof of weakening. They should be logged as watch items in the
next sub-phase plan's §3 (per axis-spec "Tier 2 — log to retro §3 'watch
items'"), not folded into brief scope.

### Axis 2 — 20 callsites of 3-token append reborrow

The 3-token `&mut conn.into()` form coexists with the 4-token canonical
`&mut (&mut *conn).into()` documented at `governance_log.rs:272`. Both
forms produce identical `DbPool::Conn` (verified via
`diesel_utils/src/connection.rs` `From` impls).

Affected files (callsite counts):

- `submit_jury_vote.rs` (9) — lines 250, 400, 452, 546, 679, 696, 772,
  875, 924. Notable: the same file uses canonical 4-token for
  `config::get_int` and `actor_pseudonym_helper::get_or_create` calls.
- `admin_assign_jury.rs` (4) — lines 237, 276, 290, 982. Line 1212 (from
  v1-JM-d, a later sub-phase) uses canonical 4-token. **Pattern was
  upgraded but not back-filled.**
- `admin_emergency_remove.rs` (4) — lines 243, 280, 295, 319.
- `admin_close_case.rs` (1), `admin_config.rs` (1), `admin_rule_sets.rs`
  (1) — single hits each.

Watch-item shape: this is mechanical, low-risk, and would close cleanly
in a single back-fill commit. Promotion candidate for a `chore(lint):`
follow-up post-merge of the next sub-phase, OR a one-shot mechanical-fix
fix-impl-task. Not urgent.

### Axis 3 — 1 callsite of missing-explicit-Send bound

- `inbox.rs:493` — `wrap_governance_inbound`'s generic bound is
  `A: GovernanceInboundActivity + Sync + 'a` without explicit `+ Send`.
  Functionally equivalent (`#[async_trait::async_trait]` without
  `?Send` implicitly enforces `Send + Sync`; AP framework requires
  `Send` on activity types). Sibling `send_lemmy_activity`
  (`apub/activities/src/lib.rs:122`) enumerates both `Send + Sync`
  explicitly. Watch-item: add `+ Send` for documentation parity.

### Axis 5 — 5 callsites of `let mut pool = …` variant idiom

20 canonical `let pool = &mut context.pool(); let conn = &mut
get_conn(pool).await?;` callsites coexist with 6 variants using
`let mut pool = …; get_conn(&mut pool)`.

- **High priority** (clean unnecessary divergence — single-line fix):
  - F1 `admin_dashboard.rs:68-69`
  - F5 `get_my_reputation.rs:37-38`
- **Low priority** (partial functional justification — intermediate
  `get_bool` / `get_int` calls reuse `pool` before `get_conn`):
  - F2 `admin_dashboard_html.rs:73`
  - F3 `admin_dashboard_html.rs:247`
  - F4 `admin_reputation_stats.rs:65`

The high-priority pair are clean back-fill candidates; the low-priority
trio require refactoring to use the canonical form (would need to thread
`pool` reborrow through the intermediate config calls).

## Tier-3 findings (stylistic; report-only, no escalation)

**7 Tier-3 findings across three axes.** Documented in per-axis reports
for completeness; no action recommended.

- **Axis 3 (1):** `inbox.rs:444` `GovernanceInboundActivity` trait
  declaration carries only `: Sized` as a supertrait; `Send + Sync` are
  enforced by `async_trait` macro expansion but invisible from the trait
  declaration. No in-file custom-trait sibling to compare against.
- **Axis 4 (5):** `publish_trust_attestation.rs:351` (URL composition,
  not trust boundary); `audit_projection.rs:28, 33, 38, 49` (local
  JSONB projection of trigger-managed `GovernanceLog.payload`, not
  trust boundary). The audit_projection file's own doc comment
  documents the silent-default as intentional schema-compat policy.
- **Axis 5 (1):** F6 `admin_audit_stream.rs:226-227` — variant idiom
  inside an async notification loop that uses `continue`-on-error
  instead of `?`-propagation. Contextually justified.

## Phase-6 prevention verification

The `brehon-conformance-audit` skill exists to catch the Phase-6
convention-divergence defect class (compile-clean federation handlers
diverging from same-file canonical siblings). The original triggering
defect — `receive_remote_moderation_label`'s
`.domain().map(str::to_string).unwrap_or_default()` at
`inbox.rs:~735` — is confirmed **absent** from trunk:

- Closed by `8b04e69a6` (fix-impl-3 for v1-federation-inbound-b).
- Axis-4 grep across all three roots returns zero
  `.unwrap_or_default()` hits.
- Sibling-aligned hard-error pattern at `inbox.rs:139-145` (sanction
  notice), `inbox.rs:249-257` (trust attestation), `inbox.rs:739-745`
  (moderation label, current state) all consistently use
  `.ok_or_else(|| LemmyErrorType::Unknown(...))?` at the
  `.domain()` extraction point.

The axis-3 example concern (missing `+ Sync` on
`wrap_governance_inbound`) is also confirmed fixed on trunk by
`b9691c0ab` (fix-impl-1 for v1-federation-inbound-b).

## Hypothesis discipline

Per the per-axis specs and
`feedback_verify_automated_reviewer_claims_against_compiler.md`, every
flag is a hypothesis until either (a) the compiler proves it, or (b) a
sibling diff shows new code is strictly weaker than an enforced contract.

- **Compiler oracle:** trunk cargo check + clippy passed exit 0.
  Therefore no Tier-1 finding could be raised on compile-evidence
  grounds — only on sibling-proof-of-weakening grounds.
- **Sibling-proof-of-weakening grounds:** none found. The 26 Tier-2
  findings are documented-pattern divergences with identical runtime
  semantics or implicit-equivalence guarantees (`async_trait` Send
  inference; `From` impl equivalence for the two reborrow shapes).

## Recommendations

1. **Watch-list axis 2 back-fill for next sub-phase plan §3.** A
   single `chore(lint):` follow-up commit can back-fill all 20
   3-token callsites to the canonical 4-token form. Low-risk
   mechanical edit; one cohort task. Defer to a quiet window — not
   urgent.

2. **Watch-list axis 5 F1 + F5 high-priority pair.** Same shape as
   the axis-2 back-fill; can be bundled into the same follow-up
   commit.

3. **Axis 5 F2/F3/F4 (low priority).** Don't back-fill; they have
   partial functional justification. Re-evaluate only if a future
   refactor touches the same files.

4. **Axis 3 F3-1 documentation-parity fix.** Add `+ Send` to the
   `wrap_governance_inbound` generic bound for naming-convention
   parity with `send_lemmy_activity`. One-line edit. Bundle with
   axis-2 back-fill if/when a `chore(lint):` follow-up runs.

5. **No catch-fire actions required.** All six axes pass.

## Audit artifacts

Reports (in `.claude/PRPs/reports/`):

- `conformance-audit-trunk-axis-1-2026-05-22.md`
- `conformance-audit-trunk-axis-2-2026-05-22.md`
- `conformance-audit-trunk-axis-3-2026-05-22.md`
- `conformance-audit-trunk-axis-4-2026-05-22.md`
- `conformance-audit-trunk-axis-5-2026-05-22.md`
- `conformance-audit-trunk-axis-6-2026-05-22.md`
- `conformance-audit-trunk-rollup-2026-05-22.md` (this file)

Metrics (in `.claude/PRPs/audit-metrics/`):

- `trunk-axis-2-2026-05-22.json` … `trunk-axis-6-2026-05-22.json`
  (axis 1 had zero findings; no separate JSON authored — represented
  in this roll-up).

Cargo logs (in `.claude/`):

- `build-check-trunk-2026-05-22.log` (canonical, warm baseline)
- `build-check-audit-worktree-2026-05-22.log` (worktree, cold)
- `build-clippy-audit-worktree-2026-05-22.log`
- `build-test-norun-audit-worktree-2026-05-22.log` (in flight)
