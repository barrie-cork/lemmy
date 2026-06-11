# Brief — test bm-pr (open PR phase-test → governance-v0)

## 1. Role + dispatch line

`[role:bm-task] bm-pr test — open PR phase-test → governance-v0`

Dispatcher → `branch-manager` subagent. Execute `.claude/commands/bm/bm-pr.md` step by step (Phase 1 → 7). Auto, no prompt — `bm-pr` is AUTO-class per the BM autonomy table.

## 2. Scope

Open a single PR from `phase-test` into `governance-v0` via `gh pr create --repo barrie-cork/lemmy --base governance-v0`. Title: "test — dogfood sandbox: sandbox_clamp helper (auto-phase pipeline exercise)". Assemble body from plan reference + commit log. Capture PR #/URL. Append a runlog entry at `.claude/runlog/test-runlog.md` (create if absent).

**This sub-phase (test)** is a pure dogfood sandbox — adds `crates/utils/src/sandbox.rs` (a trivial `sandbox_clamp(value: u32, max: u32) -> u32` helper) and one `lib.rs` module declaration. Goal is to exercise the full `/auto-phase` orchestration end-to-end. Validation: advisor-laptop `cargo check --workspace` passed on phase-test tip `50195e511`. validate-pending DQ `6b7c4b329002-001` resolved (`result: pass`).

Boundaries:
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `Cargo.{toml,lock}`, `rust-toolchain.toml` (BM file-ownership hard refusal).
- Do NOT post a PR comment, submit a review, or send a Telegram ping.
- Do NOT open as `--draft` (CR skips drafts).
- Do NOT merge — `bm-merge` is a separate later step behind user gate 5.
- Commit ONLY `.claude/runlog/test-runlog.md` (the Phase 6 append).

## 3. Required reading

- `.claude/commands/bm/bm-pr.md` — the operational script (Phases 1-7). Follow literally.
- `.claude/rules/branch-manager.md` — file-ownership boundaries + autonomy table.
- `.claude/rules/phase-branch.md` — PR base = `governance-v0`, not draft.
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` MANDATORY on every `gh pr` call.
- `.claude/PRPs/plans/test.plan.md` — read the H1 for the title; §4 for solution summary; §13 for task list.

## 4. Constraints

- **L14 git-sequence (load-bearing):** the Phase 6 runlog append MUST follow: **`Edit/Write .claude/runlog/test-runlog.md` → `git add .claude/runlog/test-runlog.md` → `git commit -m "chore(bm): ..."` → `git push origin phase-test`** — ONLY THEN return Phase 7 output. Do NOT `git checkout` or `git pull` after the Edit and before commit+push.
- **Commit subject:** the runlog commit MUST be `chore(bm): test PR opened #<N>`.
- **Phase 1 pre-conditions:** branch `phase-test` pushed @ `50195e511`, 7 commits ahead of `governance-v0` on daemon. Tree should be clean. If any pre-condition fails, STOP and surface.
- **Phase 1b DQ historical-fail sweep:** scan `pending[]` for `kind == "validate-pending"` with `result == null || result == "fail"`. The DQ `6b7c4b329002-001` is now in `resolved[]` with `result: "pass"` — expect "swept 0 entries". Do NOT broaden the filter.
- **Phase 1c Phase-2 e2e gate:** this phase does NOT touch `crates/server/tests/e2e.rs` — e2e gate is N/A. The plan is a `lemmy_utils` unit-test-only change. Gate PASSES trivially.
- **Retro gate:** no pre-bm-pr retro required (test sandbox; retro done separately). Expect gate passes.
- **DQ attribution:** BM NEVER writes `answered_by: "advisor"` or `"user"` — only `"bm-self-resolved"` for its own writes.
- **Linux-compile gate:** phase diff does NOT touch `Cargo.toml`, `Cargo.lock`, `migrations/**`, or `cfg(unix)` code. Gate is N/A (Option-2 scope; pure-logic Rust). Do NOT raise `validate-pending-laptop-linux`.
- **Telegram:** do NOT send a ping.

Brief commit body: authored on `governance-v0`. Mandatory file-class lessons: N/A (bm-task). §2.3 hybrid search: L14 git-sequence (PMD #115), BM file-ownership (PMD #335). L14 injected as load-bearing constraint.
