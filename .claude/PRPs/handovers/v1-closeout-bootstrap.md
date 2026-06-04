# v1 Close-Out — Lane Bootstrap Handover

> **Read this with zero conversation context.** It is self-contained.
> **Authored:** 2026-06-04 by the canonical advisor session that prepared this lane.

---

## RESUME — what to do first in this session

You are in the **`brehon-fork-closeout`** lane (CWD `C:/Users/barri/Developer/brehon-fork-closeout`, branch `phase-v1-closeout`). This lane executes the **v1 close-out plan** — cleanup/refactor of the now-shipped v1, while M1 (chat infra / "v2") runs *in parallel and untouched* in `brehon-fork-validate`.

**Plan (read it):** `.claude/PRPs/plans/v1-closeout.plan.md` (tracked; also at `C:/Users/barri/.claude/plans/steady-seeking-lampson.md`).

**Next concrete action:** start **Phase 1 — Repo & meta hygiene** (the first executable phase; Phase 0 bootstrap is already done — that's this handover). Phase 1's first step is **1a stale-worktree teardown** (a destructive `rm -rf` that needs explicit user go-ahead) OR jump to **1b plan-file archival** (the first 🟦 workflow). The plan's approval-noted **first workflow to run** is actually **Phase 3 carry-patch audit** as the small cost-gauge run — consider running that first to gauge token burn before broader workflow sweeps.

`VERIFIED_AT: 83694fed2` (governance-v0 + phase-v1-closeout tip at bootstrap).

---

## State at bootstrap (2026-06-04)

- **v1 is shipped.** Every lane in `.claude/PRPs/v1-roadmap.json` is `done` (JM, SL, AD, RT r1–r5, SR ship-1→3, federation-inbound a–e, redaction-r1, quality r1→r3c + meta-lanes).
- **M1 is IN FLIGHT** in `brehon-fork-validate` on `phase-m1-b` (Tasks 0–7, chat infra). **Do not touch it.** As of bootstrap, `phase-m1-b` had ≥5 commits not on trunk (Task 3/4 messaging-config). M1 was NOT merged.
- **DQ:** the canonical `governance-v0` DQ was clean (pending: []) at bootstrap. This lane writes its OWN `.claude/decision-queue.json` (per-worktree isolation). Do NOT write phase DQ from the canonical `brehon-fork` checkout.
- **No Junior tasks running** at bootstrap (`list_tasks(status=running)` → []).

---

## The phase split — what is safe NOW vs GATED on M1 merge

```
SAFE NOW (no crates/ mutation — zero M1 conflict):
  Phase 1  Repo & meta hygiene        🟦 workflow (audit) → 🟨 apply
  Phase 2  Doc-drift reconcile        🟦 workflow → 🟨 apply
  Phase 3  Carry-patch upstreaming    🟦 workflow (audit) → 🟨 user files PRs
  Phase 4  deps-r2 security           🟩 four-role — RUN NOW, gate only T2 (see below)
  Phase 5  deps-r3 (wasmtime)         🟨 DECISION — keep deferred + /schedule watch

GATED ON M1 MERGE (crates/ mutation — must not race phase-m1-b):
  Phase 6  e2e.rs decomposition       🟦 plan-split → 🟩 execute
  Phase 7  type-state retrofit (6)    🟩 four-role
  Phase 8  governance TODO sweep      🟦 audit → 🟩/🟨 apply + close-out retro
```

### THE M1-MERGE GATE (the prime directive — check before any Phase 6+ work)

Phases 6–8 (and Phase 4 T2, and Phase 3 `#___` backfill) touch files M1 is editing. Before dispatching ANY of them, run:

```bash
git fetch origin
git log origin/governance-v0 ^origin/phase-m1-b --oneline | head -1   # M1 merged? non-empty trunk-ahead is fine
git log origin/phase-m1-b ^origin/governance-v0 --oneline | head -1   # <-- THIS must be EMPTY = M1 merged
```

**If the second command returns ANY commits, M1 is NOT merged — do not start Phase 6/7/8 or Phase-4-T2.** When it's empty, rebase this worktree onto post-M1 `governance-v0` first, then proceed.

M1's no-touch implement-set: `crates/server/tests/e2e.rs`, `crates/api/api/src/governance/{mod,messaging_config,admin_config}.rs`, `crates/db_schema/{newtypes,…/governance/…}`, `crates/db_schema_file/src/schema.rs`, `crates/api/api_utils/src/{notify,plugins,bridge_notify,lib}.rs`, `Cargo.toml`.

---

## Approval decisions (locked 2026-06-04)

1. **Phase 4 / deps-r2** — RUN NOW, gate only T2. T1 (webmention inline, `api_utils/src/utils.rs`), T3 (tar dispute, GitHub UI), T4 (wasmtime log DQ) proceed now. T2 (`[patch.crates-io]` in `Cargo.toml`) waits for M1 merge. Brief: `.claude/PRPs/briefs/v1-deps-r2-planning-1.md` (fully scoped). Watchpoint: `rg "webmention::|send_webmention" crates/` must = 6.
2. **Phase 5 / wasmtime** — KEEP DEFERRED. Document x86_64-not-vulnerable, suppress 13 alerts in Dependabot, and **set a `/schedule` watch on `extism/extism` PR #847** to adopt the upstream release when it lands. (13 alerts are aarch64-only criticals; Brehon deploys x86_64.)
3. **First workflow** — Phase 3 carry-patch audit = the cost-gauge run.

---

## Carry-patch audit (Phase 3 — already done this planning pass)

11 `TODO(brehon-fork)` markers → **4 upstreamable, 7 fork-local**.

| Upstreamable (file the PR) | Location |
|---|---|
| Windows signal handling ×2 (`#[cfg(not(windows))]` + ctrl-c fallback) | `crates/server/src/lib.rs:55,284` → one upstream PR |
| clippy `#[expect]` removal ×2 (no-longer-firing `multiple_bound_locations`) | `crates/diesel_utils/src/pagination.rs:236` + `crates/db_views/vote/src/impls.rs:130` → second upstream PR |

Fork-local-forever (7): all `membership_state` governance coupling — `db_schema/{lib,source/person,impls/person}.rs`, `api_crud/user/create.rs`, `db_views/registration_applications/impls.rs`. Convention: `.claude/lessons/feedback_carry_patch_todos.md` (fill `#___` after filing; governance code needs no marker). **The `#___` backfill is a `crates/` edit → gated behind M1.** PR filing itself is 🟨 user action against `LemmyNet/lemmy` (NOT the fork).

---

## Harness bootstrap — DONE (verified at 83694fed2)

This lane is fully wired. All checks passed at bootstrap:
- ✓ Submodules initialized (`crates/email/translations` populated — fixes `lemmy_email` build).
- ✓ `.mcp.json` copied — HTTP PMD daemon (`http://localhost:11435/mcp`, shared cross-lane by design), `junior-brehon` (SSH homeserver), `ref-context`, `tavily`. (No `PROJECT_MEMORY_DB` env — correct under the 2026-05-30+ HTTP topology per `pmd-invariants.md` #1.)
- ✓ `.env` copied.
- ✓ `.claude/settings.local.json` copied → `pmd-canonical-guard.sh` + `session-start-multi-lane-check.sh` wired at SessionStart; `refuse-ssh-reset-hard-shared-checkout.sh` at PreToolUse (tracked `settings.json`). All 3 programmatic wiring checks returned OK.
- ✓ Hook scripts present on disk; worktree git state clean; the 3 wiring files correctly gitignored.

**On session start here, expect:** no `pmd-canonical-guard.sh` WARN (PMD path correct), and `session-start-multi-lane-check.sh` may WARN that `phase-m1-b`'s tip advanced recently — that is EXPECTED (M1 is live) and is the surface-first signal, not an error. The first user-visible response should be a one-line lane status (surface-first ritual): `lanes: brehon-fork-closeout:phase-v1-closeout active; other-active: brehon-fork-validate:phase-m1-b (M1, live)`.

---

## Isolation guarantee (the whole point)

- **Working tree:** dedicated (`brehon-fork-closeout`) — never shares files with M1.
- **Branch:** `phase-v1-closeout` — never `phase-m1-b`.
- **DQ:** per-worktree `.claude/decision-queue.json` — one writer (this lane). Canonical `brehon-fork` does `governance-v0` meta-edits only.
- **PMD:** intentionally SHARED (HTTP daemon, cross-lane by design per `pmd-invariants.md` #1) — lessons/retros are global. This is a feature (shared learning), not an isolation leak. Only the DQ + working tree are lane-private.
- **Lane mode:** A (dedicated worktree + own session).

---

## Teardown (at close-out completion)

After the close-out lane's work merges (PRs for any `crates/` phases; direct commits for meta/docs):
```bash
cd C:/Users/barri/Developer/brehon-fork
git worktree remove ../brehon-fork-closeout       # --force only if submodule worktree blocks
git branch -d phase-v1-closeout                     # after its PR(s) merged
```

## See also
- `.claude/PRPs/plans/v1-closeout.plan.md` — the full plan (8 phases, execution-lane tags, verification).
- `.claude/rules/multi-lane-worktree.md` — lane discipline + hard refusals.
- `.claude/rules/advisor-orchestrator.md` — polling loop, gates, §3.1 stage-shape.
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` — the bootstrap steps this lane completed.
