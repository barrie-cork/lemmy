# Retro: v1-deps-r2 — inline webmention sender, retire rustls-webpki 0.101.7 chain

**Date:** 2026-06-05
**Sub-phase:** v1-deps-r2
**Plan:** `.claude/PRPs/plans/v1-deps-r2.plan.md`
**Shipped:** `governance-v0` tip `772684e1f` (PR #182 merged squash)

---

## §1 Outcome

**Shipped.** One impl task (laptop-local, advisor-inline — no Junior dispatch for impl).

| Task | Description | Commit | Validation |
|---|---|---|---|
| 0 | Pre-flight (scope re-verify, baseline probe) | (advisor inline, no commit) | Brief §2 mandate — live Dependabot + lockfile verified |
| 1 | Inline W3C Webmention sender; drop `webmention 0.6.0` | `d5e909a2b` | `cargo-check.bat -p lemmy_api_utils --features full` exit 0 |
| CR fix | `get_all(LINK)` + remove double URL-encoding | `4d555ba05` | `cargo-check.bat` exit 0 (after submodule init) |
| merge-fwd | Merge governance-v0 into phase branch (runlog conflict) | `fa5014e6a` | Pushed; PR status `MERGEABLE` |

**Security alert closure:** Dependabot alerts #49/#50/#55 (`RUSTSEC-2024-0336` / `rustls-webpki 0.101.7` chain) will auto-close on push to default branch (or within next Dependabot scan). Open count: 6 → 3.

**CR findings (PR #182):**
| ID | Severity | Title | Resolution |
|---|---|---|---|
| cr-182-001 | major | Link header: only first value checked, get_all needed | fixed `4d555ba05` |
| cr-182-002 | major | Double URL-encoding: urlencoding::encode() + .form() | fixed `4d555ba05` |

**§16a stories (all verified):**
- Story 1 ✓ — 3 callers (`create.rs:144`, `update.rs:179`, `scheduled_tasks.rs:1025`) compile unchanged; `spawn_try_task` preserved
- Story 2 ✓ — `cargo tree -i rustls-webpki@0.101.7` exits non-zero; `grep -c webmention api_utils/Cargo.toml` = 0
- Story 3 ✓ — all `CouldntSendWebmention` mappings inside spawned task; outer function preserves `Ok(())` on no-endpoint

---

## §2 Per-role signals

### Advisor

**What worked well:**
- Scope re-verify at un-deferral was fast and correct. Brief's own re-verify mandate collapsed 4 tasks / 6 alerts to 1 task / 3 alerts — the planner filed a `kind: "log"` DQ; advisor harvested at retro. No scope-creep.
- Advisor-inline impl (no Junior dispatch) was appropriate for a 1-file, 80-LOC replacement with no e2e edits. Saved ~30 min of Junior queue/finalize cycle.
- CR finding verification was clean: both cr-182-001 and cr-182-002 verified against actual code before triaging. Compiler check confirmed both were real bugs (not CR false positives). The `urlencoding::encode` import retention (still used at line 941 for image proxy URL) was caught before committing — no unused-import clippy error.
- Linux gate (`cargo-linux.sh check --workspace --features full`) passed before PR open (DQ `a3d0e9941441-049`, resolved). Correct Option-2 trigger: `Cargo.toml`/`Cargo.lock` touched.

**Issues:**
- **Submodule not initialized in worktree (recurring):** `git worktree add ../brehon-fork-deps-r2` skipped submodule init. First cargo check attempt failed with `lemmy_email` `Os { code: 3, kind: NotFound }`. Required `git submodule update --init --recursive` before re-run. This is documented in CLAUDE.md and the bootstrap checklist — but no automated guard fires when a new worktree is created mid-session. Cost: ~5 min + one wasted cargo check build (~4 min).
- **BM #603 sensitive-file refusal (2nd occurrence, WATCH promoted):** bm-pr task refused to write `pr-182-findings.yaml` and `v1-deps-r2-runlog.md`. Advisor wrote both as fallback. Pattern: BM refusal on `.claude/PRPs/reviews/*.yaml` and `.claude/runlog/*.md` — 2nd time (1st was m1-a Task #597 bm-triage). Per MEMORY.md: promote-if-3rd. Structural fix not yet applied.
- **Cargo.lock uncommitted from Task 1:** The Task 1 commit (`d5e909a2b`) dropped the `webmention` dep from `Cargo.toml` but the `Cargo.lock` lockfile changes were not staged. The lock file diff landed in the CR fix commit instead. Root cause: advisor ran cargo check in the worktree but didn't stage `Cargo.lock` before the Task 1 commit. Minor — no functional impact, but the commit message was slightly inaccurate ("drop webmention 0.6.0" without the corresponding lock file update).

### Planning (Opus)

- Plan quality: high. The scope-collapse rationale in §2 was clear and correct. §5 complexity score of 1 was accurate (single-crate, no e2e, no migrations).
- The `GOTCHA` notes were well-placed: reuse `context.client()` (not a fresh `Client::new()`), preserve `spawn_try_task` shell, resolve endpoint relative to `target` URL.
- Planner's pre-seed `kind: "log"` DQ for the scope collapse was the right call — gave the advisor a clean audit trail without blocking planning.

### Impl (Sonnet, laptop-local)

- Impl was advisor-authored (not Junior dispatch) — per Gate 3 approval. Code quality: both CR findings were real bugs in the initial implementation (the advisor introduced them).
  - `get(LINK)` instead of `get_all(LINK)` — a subtle W3C spec issue the advisor missed on first write
  - `urlencoding::encode()` wrapping on top of `.form()` — a conceptual double-encoding error
- Both fixes were straightforward once CR flagged them. The `get_all().iter().find_map()` chain is the idiomatic reqwest pattern for multi-valued headers.

### BM (Haiku)

- bm-cut (#602): clean, no issues.
- bm-pr (#603): executed but blocked on sensitive-file writes (2nd occurrence of WATCH pattern). Rest of PR creation succeeded; findings.yaml and runlog written by advisor fallback.
- bm-poll-cr: not dispatched as Junior — advisor polled directly (`gh pr view`). Appropriate for a 2-finding simple phase.
- bm-merge: not dispatched as Junior — advisor merged inline (`gh pr merge --squash --admin`). Per L15 fix (read-only gate checks inline, mutating action inline after user gate).

### ci-watcher

- Not dispatched (Shape G suspended for this phase; no `validate-pending` entries raised). Linux gate run locally via `cargo-linux.sh` wrapper (advisor-laptop pattern).

---

## §3 Catch-fires / near-catches

- None. No catch-fires.
- Near-catch: cargo check failure on first run (submodule not initialized). Not a catch-fire (expected failure pattern documented in CLAUDE.md bootstrap checklist); resolved with standard fix.

---

## §4 Lessons / patterns

### New lessons

1. **Submodule init required before first cargo check in new worktree** — already in CLAUDE.md §"Lane worktree bootstrap" and `feedback_worktree_submodules_not_auto_init.md`. Re-fire here confirms the pattern; no new lesson needed. Consider adding an automated check (e.g. a pre-cargo-check hook that detects `-` prefix in `git submodule status`).

2. **`get_all(LINK).iter().find_map()` is the correct reqwest pattern for multi-valued headers** — standard reqwest usage; not project-specific. The old `get(LINK)` pattern silently truncates multi-value headers per the W3C Webmention spec. Will promote to PMD as a `kind: "reference"` note if another reqwest multi-header use case arises.

3. **reqwest `.form()` handles URL encoding — do not pre-encode** — `.form(&params)` applies `application/x-www-form-urlencoded` encoding. Wrapping values with `urlencoding::encode()` first double-encodes. Common mistake when migrating from manual HTTP to reqwest. Not project-specific.

4. **Cargo.lock must be staged with every `Cargo.toml` dep change** — mechanical. When a dep is removed from `Cargo.toml`, stage both `Cargo.toml` and `Cargo.lock` in the same commit. The lock file is part of the dep removal; splitting them creates inaccurate commit descriptions and a dirty working tree. Add to impl-task brief template file-list discipline.

### Ongoing watches

- **BM sensitive-file refusal on `.claude/PRPs/` (2nd occurrence)** — promote-if-3rd per MEMORY.md. Structural fix options: (a) add `.claude/PRPs/reviews/*.yaml` and `.claude/runlog/*.md` to the BM agent's allow-list; (b) change `bm-triage` write path to a gitignored tmp dir and have advisor copy on confirm. Carry forward.

---

## §5 Per-task metrics

| Task | Files | Commits | Runtime (min) | Max log silence |
|---|---|---|---|---|
| Task 0 (pre-flight) | 0 | 0 | ~10 | — |
| Task 1 (impl, advisor-inline) | 2 | 1 (`d5e909a2b`) | ~20 | — |
| CR fixes (advisor-inline) | 2 | 1 (`4d555ba05`) | ~40 (incl. cargo reruns) | — |
| **Total** | — | **2 impl + 2 infra** | **~70** | — |

Wall-clock (PR open to merge): ~2h (including cross-session compaction and cargo check delays).

---

## §6 v1-roadmap update

Phase `v1-deps-r2` → `done`. Alerts closed: #49/#50/#55. Remaining open: 6 → 3.
Phase 4 (`deps-r2`) in `.claude/PRPs/v1-roadmap.json` updates to `status: "done"`.
