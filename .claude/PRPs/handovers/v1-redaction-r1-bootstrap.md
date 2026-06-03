---
phase: v1-redaction-r1
plan: .claude/PRPs/plans/v1-redaction-r1.plan.md
phase_branch: phase-v1-redaction-r1
worktree: C:/Users/barri/Developer/brehon-fork-redaction-r1
authored: 2026-06-01
authored_by: advisor (canonical brehon-fork / governance-v0 session, phase transition)
purpose: Bootstrap the v1-redaction-r1 advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon v1-redaction-r1.** This session drives from the lane worktree `C:/Users/barri/Developer/brehon-fork-redaction-r1` on branch `phase-v1-redaction-r1`. Meta-edits (briefs, lessons, runlog on governance-v0) live in the canonical `brehon-fork` checkout.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm lanes.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `851ce382d`; if drifted, check log.
3. Check DQ on the phase branch: use `bash scripts/brehon/resolve-dq-canonical.sh v1-redaction-r1` from the lane worktree, or read `phase-v1-redaction-r1` tip DQ via the tmpfile pattern. At handoff: **0 pending entries**.
4. Check `cat .claude/governance-log/retro-bypass.jsonl 2>/dev/null | tail -5` for hook fail-open events.
5. Check Telegram completion hook: `mcp__junior-brehon__list_hooks` — hook ID 1 must exist; recreate if absent per `feedback_daemon_telegram_completion_hook.md`.

## Current stage

**Stage: bm-pr DONE → CodeRabbit pending → bm-poll-cr → bm-triage → merge-forward needed (CONFLICTING) → bm-merge.**

- **PR #173** is OPEN on `barrie-cork/lemmy` (`phase-v1-redaction-r1` → `governance-v0`).
- PR is **CONFLICTING** (`mergeStateStatus: DIRTY`) — v1-quality-r3c (PR #172, merged `c1ba108dc`) landed after the phase branch was cut. A merge-forward is required before bm-merge.
- DQ: 0 pending on the phase branch.
- e2e: already passed — DQ `9f1d7e7ca817-001` resolved at `cbf01834a` (advisor-laptop PASS).
- `/brehon-verify` already passed: 4/4 stories ✓ (commit `02c4167c1`).
- bm-pr-1 brief already written at `5ca62ccee`.

## Next concrete action

1. **Poll CodeRabbit** — dispatch `bm-poll-cr 173` to check if CR has posted findings.
2. **If CR findings present** — dispatch `bm-triage 173` → gate 3 (CR triage approval).
3. **Merge-forward** — from the lane worktree `brehon-fork-redaction-r1`:
   ```bash
   git fetch origin governance-v0
   git merge origin/governance-v0
   # resolve any conflicts (DQ json most likely)
   git push origin phase-v1-redaction-r1
   ```
4. **After merge-forward + CR triage clear** — gate 5 (merge confirm) → dispatch `bm-merge 173`.
5. **Post-merge** — author retro → gate 6 → `/brehon-phase-transition v1-redaction-r1 <next>`.

Do NOT dispatch bm-merge while PR is CONFLICTING.

---

## 1. v1-redaction-r1 in one paragraph

v1-redaction-r1 ships GDPR-compliant identifier scrubbing for the Brehon governance log. The deliverables: `scrub_json` recursion-depth cap (prevents stack overflow on deeply-nested objects), `actor_pseudonym` table write path hardened against PII leakage in error cases, and documentation/comment improvements in `reputation_snapshot.rs`. A fix-impl-1 cycle addressed 6 clippy lints in `reputation_snapshot.rs` after the initial impl. The phase has no new migrations; all changes are in `crates/` handler + test files.

## 2. Why v1-redaction-r1 is easier/harder than v1-quality-r3c

**Easier:** v1-redaction-r1 is handler-only — no new migrations, no config-file changes, no cross-crate schema additions. The scope is well-bounded (3 crates, 4 stories, all ✓ verified).

**Not easier:** PR #173 is CONFLICTING because v1-quality-r3c landed after this branch was cut. The merge-forward will hit DQ json conflicts (standard union-resolve). The CodeRabbit round has not completed — unknown findings count.

## 3. Lessons from v1-quality-r3c that apply to v1-redaction-r1

**Advisor-side:**
- **Daemon-local ref freshness before bm-merge dispatch** — verify `ssh homeserver "cd /srv/brehon-fork && git log governance-v0 -1 --oneline"` matches `origin/governance-v0`. If behind: `git update-ref refs/heads/governance-v0 origin/governance-v0` (never `reset --hard` — blocked by hook). See `feedback_bm_merge_daemon_local_ref_staleness.md`.
- **Falsifiable-hypothesis gate at triage** — raw-text probe is insufficient for JSON escaping bugs; use `json.load` decode test. See `feedback_verify_automated_reviewer_claims_against_compiler.md`.
- **Pre-impl HEAD check** — run `git show HEAD --stat` before dispatching any impl fix to avoid redundant rounds.

**BM-side:**
- `bm-merge` silent failure mode: task shows `done` but PR still open. Always post-condition verify with `gh pr view 173 --json state,mergedAt,mergeCommit` after bm-merge reports done.

**Impl-side:**
- `FederationInboxNonce` and other crate-internal API surfaces not in design docs cause fix-impl cascades. If the brief calls any DB model API not shown in `docs/brehon-law-inspired-network/04-data-model-and-api.md`, add a `Required reading: <source file>` line.

**Reference lessons:**
- `feedback_daemon_reset_hard_blocked_use_update_ref.md` — update-ref over reset-hard
- `feedback_bm_merge_daemon_local_ref_staleness.md` — daemon-local ref check
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — if any fix-impl cycle touches e2e.rs

## 4. v1-redaction-r1-specific watchlist

1. **`crates/api/api/src/governance/redact.rs` + `scrub_json` recursion cap** — the depth cap must be tested with a deeply-nested fixture. Story 1 checkpoint: `grep -c 'recursion_depth\|MAX_DEPTH' crates/api/api/src/governance/redact.rs` → ≥ 1.
2. **`actor_pseudonym` write path in `crates/api/`** — the hardened path must not silently swallow errors; `result?` not `result.ok()`. Story 2 checkpoint: `grep -n '\.ok()' crates/api/api/src/governance/` returns 0 in modified functions.
3. **Clippy -D warnings** — fix-impl-1 addressed 6 lints; `cargo check --workspace --features full` must exit 0. See `feedback_clippy_test_style.md`.
4. **Merge-forward DQ conflict** — the standard union-resolve: keep HEAD's `pending[]` entries; take `resolved[]` as the larger of the two sets. Verify with `python3 -c "import json, io; dq = json.load(io.open('.claude/decision-queue.json', encoding='utf-8')); print(len(dq['resolved']), len(dq['pending']))"` after resolve.

## 5. Operational rules

- **Polling cadence:** ~10 min `mcp__junior-brehon__list_tasks`.
- **Mode B:** lane is Mode B (no separate laptop session for the phase branch; all Junior dispatch uses `base_branch=phase-v1-redaction-r1`). The canonical `brehon-fork` checkout drives meta-edits.
- **Shape G:** RESIDUAL-ONLY (2026-06-01). Use laptop cargo + cargo-linux.sh + CR/Copilot for internal coverage. Shape G only for public green-check.
- **E2E:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true`. E2E_EXIT_NONZERO from bat wrapper is an artifact; test result line (`N passed / M failed / K skipped`) is authoritative.
- **validate-pending-laptop-linux gate:** the diff for this phase does NOT touch `Cargo.toml`/`Cargo.lock`/`migrations/**`/`cfg(unix)` — Option-2 scope rule says skip `validate-pending-laptop-linux`. Pure governance-logic Rust, no Linux-specific risk.
- **DQ attribution:** `chore|docs(advisor|decision-queue):` for advisor DQ commits; never `answered_by: "advisor"` from non-advisor sessions.
- **No cargo on EliteDesk** — hard rule per `project_laptop_canonical_cargo_runner.md`.
- **Six user gates:** plan approval (already passed), judgment-heavy DQ, CR triage approval (gate 3 — PENDING), Phase 2 e2e local vs dispatch (already done, PASS), merge confirm (gate 5), retro sign-off (gate 6).

## 6. What changed from v1-quality-r3c's rule set

- **`feedback_daemon_reset_hard_blocked_use_update_ref.md` (NEW)** — use `git update-ref` not `git reset --hard` to fast-forward daemon-local refs. Hard rule from r3c incident.
- **`feedback_bm_merge_daemon_local_ref_staleness.md` (NEW)** — mandatory daemon-ref freshness check before every bm-merge dispatch. Encoded in bm-task-brief template.
- **`validate-pending-laptop-linux` scope (clarified 2026-06-01)** — Option-2: raise ONLY when diff touches Cargo.toml/Cargo.lock/migrations or introduces cfg(unix)/cfg(target_os). This PR is pure governance Rust — skip the kind.

## 7. Catch-fire procedures

Universal triggers per `.claude/rules/advisor-orchestrator.md` §5.5:
- Phase branch uncommitted state when Junior reports complete.
- bm-task opens PR into `main` instead of `governance-v0`.
- `answered_by: "advisor"` in a commit whose subject is not `^(chore|docs)\((advisor|decision-queue)\)`.

Phase-specific:
- **bm-merge silent failure** — if bm-merge task shows `done` but `gh pr view 173 --json state` still shows `OPEN`, catch-fire: check daemon-local ref staleness (per `feedback_bm_merge_daemon_local_ref_staleness.md`) before re-dispatching.
- **CONFLICTING before bm-merge dispatch** — do NOT dispatch bm-merge with a CONFLICTING PR. Merge-forward first, push, re-verify `mergeStateStatus: CLEAN`.
- **CR critical finding in `bucket: fix-in-pr`** — block merge per `feedback_coderabbit_block_merge_critical.md`. Three prior catches confirm this is real.

## 8. Archive after v1-redaction-r1

Run `/brehon-phase-transition v1-redaction-r1 <next>`. The `<next>` phase is TBD — determined by `/roadmap-next` at ship time. This skill will close `workflow_state_v1_redaction_r1.md`, delete the two-ago record (`workflow_state_v1_quality_r3c.md`), create the next skeleton, write the next bootstrap, update MEMORY.md, and commit on governance-v0.

---

## Git state at handoff (captured 2026-06-01)

- governance-v0 HEAD: `851ce382d` — `docs(templates): bm-merge verb-constraint — daemon-local ref freshness check`
- phase-v1-redaction-r1 HEAD: `5ca62ccee` — `chore(advisor): v1-redaction-r1 bm-pr-1 brief — open PR phase→governance-v0`
- PR #173: OPEN, CONFLICTING (merge-forward needed)

Recent governance-v0 commits:

```
851ce382d docs(templates): bm-merge verb-constraint — daemon-local ref freshness check (v1-quality-r3c CF-3)
df7a76de1 docs(lessons): bm-merge daemon-local ref staleness + update-ref vs reset-hard (v1-quality-r3c CF-1/CF-4)
0c29b6e89 docs(retro): v1-quality-r3c retro — 3 stories ✓, 3 fix-impl cycles, bm-merge daemon-ref lesson
10ad3a14e chore(lessons): add missing frontmatter to feedback_brehon_subagent_model_effort_assignments.md
4c1d20b2e chore(bm): merge PR #172 complete — runlog COMPLETE entry
```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff — 0 pending entries on phase-v1-redaction-r1 tip 5ca62ccee)
```

## Stop-and-ask tripwires

- **Stop and ask if:** PR #173 has any CodeRabbit `severity: critical` finding in `bucket: fix-in-pr` — block merge per `feedback_coderabbit_block_merge_critical.md`.
- **Stop and ask if:** the merge-forward produces a conflict outside `.claude/decision-queue.json` (e.g. `crates/` conflict) — redaction-r1 made no changes to files that quality-r3c touched; any `crates/` conflict is unexpected and warrants investigation before resolving.
- **Stop and ask if:** `cargo check --workspace --features full` exits non-zero after merge-forward — the merge introduced a compile regression; do not proceed to bm-merge.
- **Stop and ask if:** the bm-merge task reports `done` but `gh pr view 173 --json state` returns `OPEN` — silent failure mode; diagnose daemon-local ref staleness before re-queuing.
- **Stop and ask if:** the next phase after v1-redaction-r1 is not evident from `/roadmap-next` — ask user to steer the next lane selection rather than guessing.
