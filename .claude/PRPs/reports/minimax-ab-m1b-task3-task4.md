# MiniMax-vs-Claude AB comparison — m1-b task 3 + task 4

**Captured:** 2026-06-07, during the stale-worker-branch close-out sweep.
**Why this file exists:** the two MiniMax-arm worker branches
(`#577` task-3-dtos-minimax-arm, `#579` task-4-handler-minimax-arm) were about to be
deleted as stale. They hold the only copy of the MiniMax-generated code for the m1-b A/B
trial. This artifact preserves the comparison data so the branches can be deleted
losslessly. The **canonical (Claude/Sonnet) arm won and merged** to trunk; the MiniMax
arms never merged.

**Arms:**

| Task | MiniMax arm (branch / commit) | Canonical arm (merged commit in trunk) |
|---|---|---|
| 3 — messaging-config DTOs | `#577` / `14e4b8c01` | `3dbb7bf48` |
| 4 — messaging-config handler | `#579` / `7cf96a076` | `f0adcdd72` |

**Trial status (per MEMORY.md `project_minimax_key_rotate_after_m1b_trial.md`):** the
broader MiniMax trial was logged as "0/4 arms produced code (all infra failures)." That
headline is about *dispatch/infra* (wrong-DB, missing ab-test branch, 404) — but these two
arms DID produce committable code. So this is the one place the trial yielded a real
code-vs-code signal. Treat the "0/4" as "0/4 cleanly dispatched," not "0/4 wrote code."

---

## Task 3 — DTOs (verdict: functional parity)

Both arms added the same two structs to `crates/api/api_common/src/governance.rs`:

```rust
pub struct AdminSetMessagingConfig { scope: String, key: String, value: serde_json::Value }
pub struct AdminSetMessagingConfigResponse { previous: ConfigValueWithProvenance, new: ConfigValueWithProvenance }
```

- **Identical:** field sets, field types, derive list
  (`Debug, Serialize, Deserialize, Clone, Default, PartialEq`), `#[skip_serializing_none]`,
  the ts-rs `cfg_attr` gating, and the deliberate omission of `Eq`
  (`serde_json::Value` is not `Eq` — both arms got this right).
- **Only difference:** MiniMax wrote more verbose doc-comments (referencing
  `split_typed_value`, plan §10.3, the flat `{previous,new}` rationale). Canonical was
  terser.
- **Verdict:** on a MIRROR-ref-heavy DTO task, MiniMax produced output functionally
  indistinguishable from the canonical arm. No correctness gap.

## Task 4 — handler (verdict: MiniMax has 2 real defects)

Both produced `crates/api/api/src/governance/messaging_config.rs` with
`admin_set_messaging_config` + `admin_get_messaging_config`, single-write, no transaction,
no governance_log append (all correct per plan §10.3). But the MiniMax arm has two
substantive problems the canonical arm avoided:

### Defect 1 — DROPPED the ADR-015 identity-policy gate (governance-critical)

- **Canonical** defines `fn validate_identity_policy(...)` AND **calls it** in the write
  path (`validate_identity_policy(&data)?; // §10.4 — ADR-015 pin`). This rejects any
  attempt to set a jury/appeals-scope `identity_policy` to a non-pseudonymous value —
  vacuously satisfied in M1, but the gate exists *now* so M2 can't introduce a
  non-pseudonymous default.
- **MiniMax** has neither the function nor the call. It wrote a doc-comment line
  `//! - No validate_identity_policy — that's Task 5.` — i.e. it *reasoned its way out of*
  the ADR-015 pin by deferring it, and got the deferral wrong (the gate belongs in the
  task-4 write path, not task 5). This is the failure mode that matters: a plausible-
  sounding scope decision that silently drops a hard ADR constraint.

### Defect 2 — `.expect()` on `serde_json` accessors (clippy-deny territory)

- **MiniMax** uses `value.as_bool().expect("is_boolean was true")` and four more
  `.expect(...)` calls in its helpers. The Lemmy workspace clippy config **denies
  `unwrap`/`expect`** (`feedback_clippy_test_style.md`). This arm would fail
  `cargo clippy --workspace -- -D warnings` — it was never validated (its
  validate-pending-laptop DQ entry, `cf65ce9f12a2-001`/`40416b286b86-001`, was never
  answered).
- **Canonical** uses `.ok_or_else(|| LemmyErrorType::Unknown(...))?` throughout — no
  panic-on-accessor, clippy-clean.

### Lesser divergences (style, not correctness)
- MiniMax split the JSON→column mapping into `derive_typed_columns` + `row_to_json_value`
  + `form_to_json_value` (3 helpers); canonical used `split_typed_value` + `row_to_value`
  (2 helpers, reads the written row back rather than re-deriving from the form).
- MiniMax added an `AdminGetMessagingConfig` query struct; canonical inlined the query
  params. Both compile-equivalent.

---

## What this AB pair actually tells us

1. **On pure pattern-following (DTOs, MIRROR-ref-heavy), MiniMax matched Sonnet.** Task 3
   is the profile the MiniMax trial was designed for (§0.1 qualifying criteria:
   not-e2e, MIRROR-ref-heavy, ≤2 files, cargo-gated). It passed.
2. **On handler logic with an embedded ADR constraint, MiniMax dropped the constraint.**
   Task 4 required *recognising* that the ADR-015 gate belongs in this handler despite the
   feature being vacuous in M1. MiniMax rationalised it away. That is the expensive class
   of error — not a compile failure (catchable by validation) but a silent governance
   omission (catchable only by review against the ADR).
3. **MiniMax tripped the workspace clippy-deny rule** (`expect`). Catchable by validation
   — but neither arm was ever validated, so it would have surfaced as a fix-cycle.

**Caveat on sample size:** n=2 tasks, one model pairing, one phase. This is a signal, not a
conclusion. The ADR-drop is the most decision-relevant data point because it's a
*correctness/governance* miss, not a style difference.

---

## Provenance (for re-derivation if needed)

The raw arm code was extracted from these commits before branch deletion:
- task-3 MiniMax diff: `git diff $(git merge-base <trunk> 14e4b8c01) 14e4b8c01 -- crates/api/api_common/src/governance.rs`
- task-3 canonical: `git show 3dbb7bf48 -- crates/api/api_common/src/governance.rs`
- task-4 MiniMax: `git show 7cf96a076:crates/api/api/src/governance/messaging_config.rs`
- task-4 canonical (= trunk): `git show governance-v0:crates/api/api/src/governance/messaging_config.rs`

The MiniMax-arm commits `14e4b8c01` / `7cf96a076` are reachable only while branches
`#577` / `#579` exist on origin. **Once those branches are deleted, this file is the
record.** (Full file bodies were reviewed at capture; the defects above are quoted
verbatim from the arm code.)

## Cross-references
- `project_minimax_key_rotate_after_m1b_trial.md` (PMD) — trial status, key rotation pending.
- `feedback_rolling_cumulative_trial_counter.md` — the trial-gate mechanism.
- `feedback_clippy_test_style.md` — the `expect`-deny rule MiniMax tripped.
- ADR-015 (`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`) — the
  pseudonymity pin MiniMax dropped.
- `.claude/PRPs/briefs/minimax-m27-trial-1.md` §0.1 — qualifying-task criteria.
