# Brief: m1-b BM-PR

## 1. Role + dispatch

`[role:bm-task] bm-pr m1-b — open PR phase-m1-b → governance-v0 — see .claude/PRPs/briefs/m1-b-bm-pr-1.md`

## 2. Scope

Open a PR from `phase-m1-b` → `governance-v0` on `barrie-cork/lemmy`.

**Produce:**
- PR opened (not draft) with title and body per §3 below
- PR number recorded in task output

**Do NOT:**
- Merge the PR
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `docs/`, `tests/`, `migrations/`
- Push any commits to the phase branch
- Comment on or review the PR (separate confirm-gated verb)
- Send Telegram pings (separate confirm-gated verb)

## 3. PR title + body

**Title:** `feat(messaging): governance_messaging_config table + admin config API + identity-policy validator + fire-and-forget bridge notify (M1-b Tree B)`

**Body:**
```
## Summary

M1-b Tree B (governance-side messaging config + bridge-notify wiring; Tasks 1–7). The Matrix bridge crate itself (Tree A, Tasks 8–13) lands in a later sub-phase; this PR is the in-workspace Lemmy-side surface only.

- **Task 1** — `governance_messaging_config` typed-column table migration (scope/key/value_type/value_int|bool|text, valid_from append-only audit history). Seeds 2 default rows (`messaging_enabled=false`, `identity_policy='pseudonymous'`).
- **Task 2** — `GovernanceMessagingConfig` Diesel model + `MessagingConfigId` newtype + schema; `read_current(scope, key)` accessor. InsertForm omits AsChangeset (append-only).
- **Task 3** — `AdminSetMessagingConfig{scope,key,value}` + `AdminSetMessagingConfigResponse` admin DTOs.
- **Task 4** — `admin_set_messaging_config` handler: admin gate + typed-column derive + single-write (no governance_log append, no dry-run — §10.3 + §12).
- **Task 5** — `validate_identity_policy` (ADR-015: jury/appeal scope `identity_policy` must be `pseudonymous`) + registered messaging-config routes.
- **Task 6** — `bridge_notify::notify_if_enabled`: reads `messaging_enabled`; false/absent → no-op (clean v0 posture); true → fire-and-forget POST to the bridge, log-and-swallow transport errors (NEVER fails the PM path). Called alongside `plugin_hook_notification` at notify.rs (additive, NO new hook per ADR-012). Enables `reqwest-middleware` `json` feature.
- **Task 7** — two e2e tests: `messaging_disabled_preserves_governance_posture` (clean-posture invariant: migration-seeded `messaging_enabled=false` row → bridge no-ops) + `messaging_identity_policy_rejects_jury_override` (validator rejects `identity_policy=real_name` for `jury` scope).

## Validation

All validation ran on the laptop (pre-Shape-G plan; cargo + e2e on laptop per the M1 lane decision):

- `cargo check --workspace --features full`: ✓ exit 0, 0 warnings (Finished 13m07s)
- `cargo test --no-run -p lemmy_server --test e2e`: ✓ exit 0, 0 warnings (Finished 14m50s)
- `cargo test --test e2e -p lemmy_server <2 new fns>`: ✓ 2 passed, 0 failed (45.8s + 46.2s)
- **Linux-compile gate** — `cargo-linux.sh check --workspace --features full` (Docker rust:1.95 mirror of CI): ✓ exit 0, 0 errors 0 warnings (Finished 23m28s). Migration + Cargo.toml json-feature change confirmed Linux-green.

## Plan reference

`.claude/PRPs/plans/m1.plan.md` §13 Tasks 1–7 (Tree B).

## ADRs honoured

- **ADR-012** (Extism plugin host): bridge notify is additive at the notify call-site — NO new plugin hook.
- **ADR-013** (EmergencyRemove): no change to case-status handling.
- **ADR-015** (pseudonymisation): `validate_identity_policy` enforces that jury/appeal-scope `identity_policy` may never be non-pseudonymous; migration seeds `identity_policy='pseudonymous'`.
```

## 4. Required reading

- `.claude/rules/branch-manager.md` — file ownership boundaries, autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/rules/phase-branch.md` — base MUST be `governance-v0`, not draft

## 5. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- Base branch: `governance-v0` (NOT `main`)
- Not draft (CodeRabbit skips drafts)
- No force-push
- Record PR number in task output
- Phase branch tip at dispatch time: `2e1972045`
