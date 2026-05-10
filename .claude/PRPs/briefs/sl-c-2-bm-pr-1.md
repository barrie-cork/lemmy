# [role:bm-task] sl-c-2-bm-pr-1 — open PR for phase-v1-SL-c-2

## 1. Role + dispatch

`[role:bm-task] sl-c-2-bm-pr-1 — open PR for phase-v1-SL-c-2 into governance-v0`

Run `/bm-pr` per `.claude/commands/bm/bm-pr.md`.

## 2. Scope

Open a pull request from `phase-v1-SL-c-2` into `governance-v0` on
`barrie-cork/lemmy`. Not a draft — CodeRabbit skips drafts.

**Deliverable:** PR open, URL recorded in runlog.

### Phase summary

v1-SL-c-2 — sponsor liability grace check, cycle 2. Five e2e test stubs
added to `mod v1_sl_c_fixtures` in `crates/server/tests/e2e.rs`:

1. `grace_check_fires_expired_case` (Task 1 — fire path, LemmyResult<()> Case A)
2. `grace_check_escapes_case_when_sponsor_revoked_after_decided_at` (Task 2 — escape path)
3. `grace_check_no_op_when_grace_expires_at_in_future` (Task 3 — no-op / future grace)
4. `grace_check_per_case_isolation_skips_bad_case_processes_good_case` (Task 4 — per-case isolation)
5. `grace_check_batch_size_config_caps_iteration` (Task 5 — batch_size config knob)

All use `LemmyResult<()>` return type (Case A — uniform with existing sibling mods).
Phase-1 workspace-checks (DQ #172-174) and Phase-2 e2e (DQ #175, 76/0 passed) all result=pass.

### PR body

```
## Summary
- Add 5 e2e test stubs for v1-SL-c-2 (sponsor liability grace check, cycle 2)
- All stubs in `mod v1_sl_c_fixtures` in `crates/server/tests/e2e.rs`
- Uniform `LemmyResult<()>` return type (Case A) throughout the mod
- Tests: fire path, escape path, no-op (future grace), per-case isolation, batch_size config

## Validation
- Phase-1 workspace-checks: DQ #172, #173, #174 — all result=pass
- Phase-2 e2e: DQ #175 — 76 passed, 0 failed, 3 ignored (1541s)

## Plan reference
`.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md`
```

## 3. Required reading

- `.claude/commands/bm/bm-pr.md` — full bm-pr procedure
- `.claude/rules/branch-manager.md` — autonomy bounds + file ownership
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/rules/phase-branch.md` — base must be `governance-v0`, not `main`

## 4. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- Base: `governance-v0` (never `main`)
- NOT a draft
- Append PR URL + number to `.claude/runlog/bm-runlog.md`
- Do NOT merge, do NOT post CR comment, do NOT send Telegram ping
