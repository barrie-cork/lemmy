# Brief: v1-quality-r3c-impl-2

## 1. Role + dispatch line

`[role:impl-task] v1-quality-r3c-impl-2 — sponsor-allowlist e2e sweep — see .claude/PRPs/briefs/v1-quality-r3c-impl-2.md`

## 2. Scope

**Produce:** extend the `all_mvp_endpoints_return_non_404` test array in
`crates/server/tests/e2e.rs` with 2 sponsor-allowlist entries (Issue #166).

**Exactly one file to edit:** `crates/server/tests/e2e.rs`

**Commit subject:** `test(e2e): add sponsor-allowlist routes to HTTP-path sweep (Issue #166)`

**Do NOT:**
- Add a second `rate_limit.set_config` call — one already exists at e2e.rs:4271-4282
- Edit any `crates/**` files other than `crates/server/tests/e2e.rs`
- Edit any `migrations/**`, `docs/**`, or `.claude/**` files
- Run `cargo check` or `cargo test` — write the `validate-pending-laptop` DQ entry and stop

## 3. Required reading

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — LemmyResult/? test return shape
- `.claude/lessons/feedback_async_pool_test_pattern.md` — async pool pattern for e2e tests
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate verbatim anchors before any Edit; run `grep -c '<anchor>' crates/server/tests/e2e.rs` and confirm = 1 before using
- `.claude/lessons/feedback_rate_limit_debug_config_post_bucket.md` — POST bucket is 6/300s; set_config already present; do NOT add a second one
- `.claude/PRPs/plans/v1-quality-r3c.plan.md` — §13 Task 2 spec

## 4. Constraints

**Pre-locate gate (mandatory before Edit):** confirm this anchor appears exactly once:

```
      "/api/v4/governance/admin/reputation-stats",
      "",
      &[200, 400, 401],
    ),
  ];
```

Run: `grep -c 'governance/admin/reputation-stats' crates/server/tests/e2e.rs`
Expected: 2 (one array entry at ~line 4225, one `.uri()` call at ~line 4366)
The array-closing anchor above (with `  ];` on the last line) is count=1 — confirmed safe.

**Edit target:** insert BEFORE the `  ];` closing line of the endpoints array (after the reputation-stats tuple). The full `old_string` to match:

```rust
      "/api/v4/governance/admin/reputation-stats",
      "",
      &[200, 400, 401],
    ),
  ];
```

**new_string** (append the two sponsor-allowlist entries, keep `  ];`):

```rust
      "/api/v4/governance/admin/reputation-stats",
      "",
      &[200, 400, 401],
    ),
    (
      "POST",
      "/api/v4/governance/admin/sponsor-allowlist/add",
      r#"{"actor_id":1}"#,
      &[200, 400, 401, 403, 422],
    ),
    (
      "POST",
      "/api/v4/governance/admin/sponsor-allowlist/remove",
      r#"{"actor_id":1}"#,
      &[200, 400, 401, 403, 422],
    ),
  ];
```

Routes are registered at `crates/api/routes/src/lib.rs:526-528` — read-only reference, do not edit.

**After the edit:** write a `validate-pending-laptop` DQ entry with:
```json
{
  "kind": "validate-pending-laptop",
  "from": "impl",
  "commands": ["./scripts/brehon/cargo-check.sh --workspace --features full"],
  "branch": "phase-v1-quality-r3c",
  "phase_task": 2
}
```
Commit + push the DQ entry, then **stop**. Do NOT run `cargo-check.sh` yourself.

**Attribution:** commit as `solo-dev <112749825@umail.ucc.ie>`. Never write `answered_by: "advisor"`.

**Mid-task push:** after every DQ write, commit + push immediately to `phase-v1-quality-r3c` so the advisor's polling loop sees it.
