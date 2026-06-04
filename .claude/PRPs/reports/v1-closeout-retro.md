# v1 Close-Out — Capstone Retro

**Branch:** `phase-v1-closeout` → `governance-v0`
**Commit range:** `0f9cdb407`..`148bcd011` (42 commits)
**Phases executed:** Phase 0 (harness) · Phase 1 (hygiene) · Phase 2 (doc-drift) · Phase 3 (carry-patch) · Phase 5 (wasmtime decision) · Phase 6 (e2e decomposition) · Phase 7 (type-state retrofit) · Phase 8 (TODO sweep)
**Phase 4 status:** DEFERRED (NO-ELITEDESK during M1; deps-r2 carries to post-m1-a)
**Authored:** 2026-06-05 (spans 2026-06-04 → 2026-06-05)

---

## §1 What shipped

| Phase | Deliverable | Key commits |
|---|---|---|
| 0 | Lane bootstrap, harness signed off, audit flag SET | `0f9cdb407` |
| 1a | 46 shipped sub-phase plans archived to `completed/` | `c715d6e7d` |
| 1b | 107 legacy integer-id DQ entries archived | `865642e8e` |
| 1c | 2 missing always-load-rule citations reconstructed | `b60d99a3a` |
| 1d | Phase 3 carry-patch audit → 2 upstream PR drafts | `5d89ccb5f` |
| 2 | SL entry-kind `(pending)` markers flipped; RT-r2 retro path corrected | `bb8a92ddd` |
| 5 | 12 wasmtime Dependabot alerts dismissed; ADR-012 deferral documented | `e719081b2` |
| 6 | e2e.rs 18,582-line monolith → 9 per-domain modules (135 tests; GOLDEN_INVARIANT `130 passed, 5 skipped`) | `5dfcb00b5`..`0f3531c81` |
| 7 | 6 governance handler sites retrofitted with `GovernanceCase<S>` phantom type-state wrapper (236-line scaffold + 6 handler edits) | `ca1bfeab3` + `9d0048c16` |
| 8 | 12 governance TODO markers cleared / reclassified; SL-b timing flake annotated | `148bcd011` |

**Phases NOT executed in this lane (by design):**
- Phase 4 (deps-r2): blocked by NO-ELITEDESK; carries to post-m1-a session.
- Phase 1c (MEMORY.md milestone-prune): deferred (lane has its own DQ; prune from canonical).

---

## §2 Per-phase complexity signal

| Phase | Files changed | Commits | Key risk |
|---|---|---|---|
| 6 | 10 (e2e.rs + 9 new modules) | 4 impl + 1 plan | Single-binary contract — any second `tests/*.rs` file breaks build |
| 7 | 9 (state.rs NEW + 6 handlers + mod.rs + lesson) | 2 impl + 3 plan/docs | Site 6 success-not-error semantics (ActiveVoteResult sentinel) |
| 8 | 9 (comment-only) | 1 | Low — zero behaviour change |
| 1–5 | ~30 (docs/meta only) | 33 | DQ archive migration; wasmtime ADR alignment |

---

## §3 Four-role retro signals

### Advisor role
- **What worked:** Workflow fan-out for Phase 6 manifest + Phase 8 TODO audit — parallel agents returned structured ledgers in one pass without burning advisor context on sequential reads.
- **What strained:** Phase 7 required careful pre-read of the site-6 semantics before dispatch. The `ActiveVoteResult` decision was non-mechanical (a plain `TryFrom` would have introduced a 200→404 regression). The advisor correctly identified this in the recon doc before any code ran.
- **Process observation:** The Phase 6 `--no-fail-fast` run revealed the 12-tests-not-run omission; always run with `--no-fail-fast` when verifying GOLDEN_INVARIANT.
- **Retro-bypass.jsonl:** not checked this session (no retro-check hook failure observed; rate trend stable).

### Planning role
Not dispatched in this lane (no Junior tasks). Closeout ran advisor/laptop-local throughout due to NO-ELITEDESK directive.

### Impl role
Not dispatched in this lane (all code was written directly in the advisor session).

### BM role
Not dispatched in this lane. No PR opened yet for Phase 6/7/8 (comment-only Phase 8 changes are `crates/` touched; PR required per `phase-branch.md`).

**Pending:** open PR for `phase-v1-closeout` → `governance-v0`. This retro ships first; PR follows.

---

## §4 MiniMax trial (accumulated count from plan §3.5a table)

No impl-task briefs dispatched in this lane — MiniMax trial counter unchanged from last phase.

---

## §5 Lessons harvested

| Lesson | Category | Action |
|---|---|---|
| Phase 6 single-binary contract: only one `tests/*.rs` root file; all modules under `tests/e2e/` via `#[path]` or `mod` | Code invariant | Documented in Phase 6 plan "LOAD-BEARING split rules"; in lesson corpus |
| CaseStatus variant count was 13 in the lesson — actual count is 12 | Stale metric | Fixed in `feedback_governance_type_state_handlers.md` |
| Workflow fan-out for comment-cluster classification needs grep verification before acting on "stale/keep" verdicts | Advisor discipline | Documented: federation cron TODO was flagged stale by workflow, confirmed by grep |
| Phase 7 site-6: `TryFrom` returning `Err` on terminal states flips 200→404 — use sentinel enum pattern | Rust type-state | Documented in lesson §"Site 6 pattern" |
| Phase 8 merge-1b TODO audit: typed protocol structs landed in `aa159a0f4` but the TODO text still said "once Agent B lands" — mismatch between code reality and comment. Workflow caught it; grep confirmed | Comment drift | Lesson implicit in Phase 8 commit message |

---

## §6 Known carry-forwards into M1/post-close

| Item | Owner | Trigger |
|---|---|---|
| Phase 4 deps-r2 (6 Dependabot alerts, webmention inline, mdurl fork) | Next close-out session | m1-a ships, daemon frees |
| SL-b timing flake: `calculated_at >= test_start` is theoretically safe but races PG↔Rust clock at sub-ms | Optional v2 hardening | Add a small tolerance or use `test_start - 1s` lower bound |
| `v2-cleanup` markers (merge-1b → typed AP protocol structs) | v2-messaging | M1 A Tree ships matrix bridge |
| v2-deferred: `majority_revocation` threshold (baseline_sponsor_count column) | v2-SL | v2 sponsor-liability sub-phase |
| v0-polish `#[ignore]` tests (GH #42, #43, #45) | v2 or dedicated deflake pass | After v2 federation-inbound-b lands (GH #43) |
| MEMORY.md milestone-prune (currently 202 lines, over budget) | Next canonical session | `memory-prune` skill |
| 2 upstream PRs to file against LemmyNet/lemmy (Windows signal handling + clippy `#[expect]`) | User action | User files PRs from prepared branches |

---

## §7 v1 formal closure

**v1 Brehon governance platform is closed.** All 11 v0 endpoints are live. v1 sub-phases SL, JM, RT, AD, federation-inbound are merged. The close-out lane (Phases 0–3, 5–8) has completed all deliverables that were executable with NO-ELITEDESK in force.

The `phase-v1-closeout` branch is ready for PR → `governance-v0`.

Next milestone: **M1** (Matrix bridge — Tree A `services/bridge/` crate, Tasks 8–14).
