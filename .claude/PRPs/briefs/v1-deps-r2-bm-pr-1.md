---
role: bm-task
verb: bm-pr
phase: v1-deps-r2
pr_number: null
created: 2026-06-05
related_dq: null
---

# [role:bm-task] v1-deps-r2 bm-pr — open PR for phase-v1-deps-r2 into governance-v0

## §1 Role + dispatch

`[role:bm-task] v1-deps-r2 bm-pr — open PR for phase-v1-deps-r2 into governance-v0`

Actual create-task description (single line, <100 chars):

```text
[role:bm-task] v1-deps-r2 bm-pr — see .claude/PRPs/briefs/v1-deps-r2-bm-pr-1.md
```

## §2 Scope

Run `bm-pr` for `phase-v1-deps-r2`. v1-deps-r2 is the **deferred Phase 4 security alert sweep** — inline the W3C Webmention sender, drop `webmention 0.6.0`, eliminating the `rustls-webpki@0.101.7` → `rustls@0.21.12` → `hyper-rustls@0.24.2` → `reqwest@0.11.27` → `webmention@0.6.0` chain. Closes Dependabot alerts #49/#50/#55 (one HIGH: RUSTSEC-2024-0336).

1 impl task delivered:
- `crates/api/api_utils/src/utils.rs` — inline `send_webmention` using `context.client()` (reqwest 0.13.2, ring provider); Link header endpoint discovery per W3C spec §3.1.2
- `crates/api/api_utils/Cargo.toml` — drop `webmention = "0.6.0"`

**Phase branch:** `phase-v1-deps-r2`
**Tip:** `d5e909a2b` (`feat(security): inline W3C Webmention sender, drop webmention 0.6.0`)
**Base:** `governance-v0`
**Repo:** `barrie-cork/lemmy`

**Open the PR only. Do NOT merge** (merge is behind user gate 5).

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` — bm-pr operational script (follow phases 1→7 verbatim)
- `.claude/rules/branch-manager.md` — BM autonomy bounds + file-ownership
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
- `.claude/rules/phase-branch.md` — phase-branch + PR flow (base `governance-v0`, never `main`; not draft)
- `.claude/PRPs/plans/v1-deps-r2.plan.md` — plan (for PR body §Summary reference)
- `.claude/PRPs/briefs/m1-a-bm-pr-1.md` — canonical-sibling brief (mirror §1–§7 shape)

## §4 Constraints

- **`--repo barrie-cork/lemmy`** on every `gh` command.
- Base: `governance-v0` (NOT `main`). Head: `phase-v1-deps-r2`.
- **Not draft** (CodeRabbit skips drafts).
- **PR title:** `Phase v1-deps-r2 — security: inline webmention sender, drop rustls-webpki 0.101.7 chain`
- **PR body** assembles from (in this order):
  - `## Summary` — v1-deps-r2 closes 3 Dependabot alerts (#49 rustls 0.21.12 RUSTSEC-2024-0336, #50 rustls-webpki 0.101.7, #55 reqwest 0.11.27 via hyper-rustls 0.24.2) by inlining a minimal W3C Webmention sender in `send_webmention` using the existing `context.client()` (reqwest 0.13.2, ring provider already installed) and removing `webmention = "0.6.0"` from `lemmy_api_utils/Cargo.toml`. Link-header endpoint discovery per W3C spec §3.1.2. No endpoint → silently Ok(()). Callers (create.rs, update.rs, scheduled_tasks.rs) unchanged.
  - `## Plan reference` — `.claude/PRPs/plans/v1-deps-r2.plan.md` Task 1
  - `## Validation` — `cargo check --workspace --features full` EXIT 0. `cargo tree -i rustls-webpki@0.101.7` EXIT 101 ("did not match any packages"). Linux compile (`cargo-linux.sh check --workspace --features full`) EXIT 0 (Cargo.toml diff triggers Phase-1d gate; result: validate-pending-laptop-linux DQ resolved pass). Callers compile unchanged (no signature change). No e2e edits; Phase 2 e2e not required.
  - `## Commits` — two commits:
    - `d5e909a2b feat(security): inline W3C Webmention sender, drop webmention 0.6.0`
    - `cb7f692d9 chore(bm-task): log branch cut phase-v1-deps-r2 post-planning approval`
  - `## Closes` — Dependabot alerts #49, #50, #55 (resolved when `webmention 0.6.0` dep removed; three downstream RUSTSEC-2024-0336 entries vanish from dep graph)

- **Phase 1b (DQ historical-fail sweep):** run it. DQ pending=0 on tip `d5e909a2b` expected. If sweep finds anything → STOP + `kind:"blocker"`.
- **Phase 1c (Phase 2 e2e gate):** NOT applicable — zero `crates/server/tests/e2e.rs` edits. Skip silently.
- **Phase 1d (Linux-compile gate):** the diff DOES touch `Cargo.toml`. Check for `validate-pending-laptop-linux` DQ at `result:pass` on `phase-v1-deps-r2`. If found: proceed. If absent: STOP + raise `kind:"blocker"` DQ noting "Phase-1d Linux gate not yet resolved — advisor running cargo-linux.sh".
- Write `.claude/PRPs/reviews/pr-<N>-findings.yaml` shell per `.claude/PRPs/reviews/SCHEMA.md`.
- Append bm-pr "PR opened" runlog entry to `.claude/runlog/v1-deps-r2-runlog.md`.
- Do **NOT** post a Telegram ping, post PR comments, or merge.
- Do **NOT** touch `crates/**`, `migrations/**`, `tests/**` (BM file-ownership HARD boundary).

## §5 Validation gates

- Phase 1 pre-conditions: branch is `phase-v1-deps-r2`, pushed, clean tree, 2 commits ahead of `governance-v0`, no existing PR.
- Phase 1b: no-op sweep (DQ pending=0 expected).
- Phase 1c: skip (no workspace e2e edits).
- Phase 1d: `validate-pending-laptop-linux` DQ must be at `result:pass`. STOP if absent.
- Phase 2: PR body assembles cleanly from §4 ingredients.
- Phase 3: `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-deps-r2 --title "<§4 title>" --body-file <tempfile>` returns PR URL.
- Phase 5: findings.yaml shell written at `.claude/PRPs/reviews/pr-<N>-findings.yaml`.
- Phase 6: runlog appended to `v1-deps-r2-runlog.md`.
- Phase 7: return PR URL + "Next suggested: bm-poll-cr after ~5-15min for CodeRabbit findings".

## §6 Expected output (return to advisor)

```text
## bm-pr complete — PR #<N> opened on phase-v1-deps-r2

**PR URL:** https://github.com/barrie-cork/lemmy/pull/<N>
**Title:** Phase v1-deps-r2 — security: inline webmention sender, drop rustls-webpki 0.101.7 chain
**Base:** governance-v0
**Head:** phase-v1-deps-r2 @ d5e909a2b
**Phase 1b sweep:** no-op (DQ pending=0, expected)
**Phase 1c e2e gate:** skipped (no e2e edits)
**Phase 1d Linux gate:** resolved (validate-pending-laptop-linux at result:pass)
**findings.yaml:** .claude/PRPs/reviews/pr-<N>-findings.yaml (shell only)
**Next suggested:** bm-poll-cr after ~5-15min for CodeRabbit findings
```

## §7 Hard refusals (per bm-task-brief.template.md §4)

1. Never touch `crates/`, `migrations/`, `tests/`, `docs/brehon-law-inspired-network/`, `.claude/PRPs/plans/**`, `Cargo.lock`, `rust-toolchain.toml`.
2. Never post a PR comment without advisor confirmation.
3. Never submit a PR review.
4. Never merge this PR.
5. Never force-push.
6. Never delete a branch.
7. Never send a Telegram ping without confirmation.
8. Never write `answered_by: "advisor"` or `approved_by` in any DQ entry.
9. Always use `bash scripts/brehon/dq-v3-new-entry.sh` for new DQ ids.
10. `--repo barrie-cork/lemmy` mandatory on every `gh` command.
11. Base must be `governance-v0`.
12. PR must not be draft.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `phase-v1-deps-r2`). Brief will be committed on `governance-v0` after Linux gate passes + advisor switches back to governance-v0. bm-task worker uses `base_branch=governance-v0`._
