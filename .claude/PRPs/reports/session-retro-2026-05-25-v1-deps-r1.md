# Session retro — v1-deps-r1

**Date:** 2026-05-25
**Phase branch:** `phase-v1-deps-r1`
**PR:** #153 merged `ab1e79a70` → `governance-v0` at 2026-05-25T11:47:52Z
**Plan:** `.claude/PRPs/plans/v1-deps-r1.plan.md`
**Roles:** advisor (laptop) + impl-task (Sonnet 4.6, Junior) + bm-task (Haiku 4.5, Junior)

---

## §1. Phase summary

Bundled 12 Cargo dependency bumps into 3 tasks to amortise cargo + e2e validation cost:

- **Task 1** (`7bd047f2d`): diesel-async 0.8 → 0.9 — wrapper trait bound rewrite (`AsyncFnOnce + AsyncFunc`) + 42-callsite closure-shape sweep + 30 `scoped_futures` import collapses across 31 files
- **Task 2** (`16d722c8e`): sha2 0.10 → 0.11 — Cargo pin bump; zero consuming-code changes (API preserved; ADR-012 hash chain byte-identical)
- **Task 3** (`939868483`): 8 SemVer-compat bumps — serde_with 3.20, bcrypt 0.19.1, tokio 1.52.3, rustls 0.23.39, html2text 0.17.1, jsonwebtoken 10.4.0, lettre 0.11.22, rss 2.0.13

Validation: `cargo check --workspace --features full` ✓, `cargo clippy --workspace --features full --no-deps -- -D warnings` ✓, phase-tip e2e 109 passed / 0 failed (36 min, 2026-05-25).

---

## §2. Per-task metrics

| Task | Files | Commits | Runtime (min) | Max log silence (min) | Notes |
|------|-------|---------|---------------|----------------------|-------|
| T1 diesel-async 0.9 | 33 modified | 1 (+3 DQ/validate) | ~90 | ~15 | Worker #456 forked stale ref (#292 incident) — cherry-pick recovery |
| T2 sha2 0.11 | 2 modified | 1 (+2 DQ/validate) | ~30 | ~10 | Smooth |
| T3 SemVer bundle | 9 Cargo.toml modified | 1 (+1 DQ/validate) | ~25 | ~8 | dashmap 6.2.x + rustls 0.23.40 not published; WP-6 compliant at 6.1.0/0.23.39 |
| **Phase total** | — | **3 impl + 9 advisor/DQ** | **~3 days elapsed** | — | Multi-session, Shape G suspended |

---

## §3. Four-role retro signals

### Advisor
- **What worked:** validate-pending-laptop discipline clean across all 3 tasks; handover trailer injection before T3 worked (d70a320f0); `/brehon-verify` caught the doc-comment advisory (Story 1, `connection.rs:83`) cleanly.
- **Watch:** the `chore: restore stashed changes from earlier session` commit (`17b40e2a4`) pattern is dangerous — it reintroduced 2 resolved DQ entries into pending on governance-v0, causing bm-merge (#461) to false-block. The stash-restore commit should never carry DQ state.
- **Process miss:** bm-merge brief §3 said "findings YAML required" but it's gitignored. Brief v2 explicitly noted this — good retrofit.

### Planning (Junior, Opus 4.7)
- T2 PR #292 stale-base recovery required cherry-pick (`feedback_junior_292_stale_base_recover_recipe.md` applied).
- Plan §16a stories well-specified — all 4 verified cleanly. Advisory doc-comment miss (Story 1) was non-blocking and correctly classified.

### Impl (Junior, Sonnet 4.6)
- T1: correct enumeration of 42 callsites (plan said 42; worker confirmed 42 + 1 comment-only). Clean.
- T2: zero consuming-code changes — correctly identified sha2 0.11 API preservation.
- T3: correctly identified 2 unavailable packages (dashmap 6.2.x, rustls 0.23.40) and substituted latest stable with HANDOVER documentation.
- Advisory miss: `crates/diesel_utils/src/connection.rs:83` doc-comment referencing `scope_boxed()` — plan §12 required deletion but worker left it. Non-blocking; carry-forward to `chore(lint):` if desired.

### BM (Junior, Haiku 4.5)
- bm-pr, bm-poll-cr, bm-triage all clean.
- bm-merge false-blocked on stale DQ state (see Advisor §3 above). Retry (#462) succeeded.
- bm-merge brief should pre-clear Gate 1 (findings YAML absent = gitignored) more explicitly — done in v2 brief.

---

## §4. CR triage summary

9 findings — 0 fix-in-pr, 4 rebut, 5 wont-fix:

| ID | Severity | Bucket | Reason |
|----|----------|--------|--------|
| cr-1 | major | rebut | Plan file exists; CR misread diff scope |
| cr-9 | major | rebut | Pre-existing `registration_created` logic; out of deps-r1 scope |
| cr-2 | medium | rebut | DQ historical audit record; retroactive edits are process breaches |
| cr-3 | medium | rebut | Pre-existing lemmy_apub failure; not a contradiction |
| cr-7 | medium | wont-fix | Debug artifact; no build/runtime impact |
| cr-4,5,6,8 | low | wont-fix | MD031/MD040 noise on advisor brief/handover meta-files |

---

## §5. Watch items

1. **`chore: restore stashed changes` commits on governance-v0 carry DQ state** — the bm-poll-cr finalize stashed uncommitted state (including a stale DQ) and restored it after merge, reintroducing resolved entries. The bm-task finalize stash/restore pattern needs a DQ-state sanitisation step, or the advisor should verify the DQ is clean on governance-v0 before dispatching bm-merge. → File as lesson candidate if observed again.

2. **`git update-ref` vs `git merge --ff-only` for daemon governance-v0 sync** — when daemon is checked out on governance-v0 (post-finalize), `git fetch origin X:X` is blocked. Correct fallback: `git merge --ff-only origin/governance-v0`. Precheck Check 3b instruction should note this case. → Codify if 2nd occurrence.

3. **Advisory doc-comment (`connection.rs:83`)** — plan §12 required deletion of `/// \`|conn| async move { ... }.scope_boxed()\`` from `run_transaction` doc-comment. Worker left it. Non-blocking. Fold into a future `chore(lint):` or next phase touching `diesel_utils`.

---

## §6. Lessons promoted this phase

- `feedback_junior_292_stale_base_recover_recipe.md` applied (T2 cherry-pick recovery).
- Emerged: stash-restore DQ contamination pattern (§5 item 1 above — not yet promoted; watch for recurrence).
