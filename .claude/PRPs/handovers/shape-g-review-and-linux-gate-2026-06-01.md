# Handover — Shape-G review + Linux-compile gate (2026-06-01)

**Author:** advisor session (canonical `brehon-fork` / `governance-v0`)
**Stop reason:** user asked to stop at a safe point; remaining validation + work continues in a new session.
**Safe-stop state:** all work COMMITTED + PUSHED to `origin/governance-v0`. Working tree has only BM-session files dirty (not mine). No uncommitted work of mine.

---

## What this session did (all shipped to governance-v0)

Three pushed commits, in order:

1. `7a4dfecb8 chore(ci): harden Shape-G workflow triggers — belt-and-braces dispatch-only`
2. `d2a33d39a docs(ci): add Copilot review instructions — residual lane only`
3. `88d71620d docs(lessons): verify branch diff-vs-trunk before concluding code is missing`
4. `224606866 feat(ci): embed cargo-linux.sh as a diff-scoped bm-pr gate` ← the big one

### Theme 1 — Shape-G "sensible config" (DONE, verified)
- **Billing reset confirmed by user:** 24 / 3,000 min. The May-31 adr-compliance "payment failed / spending limit" failures were pre-reset exhausted-minutes, NOT a defect. (gh token lacks `user` scope → can't read billing via API; `gh auth refresh -h github.com -s user` to fix.)
- **`cargo-linux.sh` VERIFIED working** (ran `cargo 1.95.0` in `rust:1.95` container, exit 0). The memory claiming it was a "deferred TODO" was stale — corrected in `project_laptop_canonical_cargo_runner.md`.
- **Workflows hardened** (`7a4dfecb8`): `cargo-validate-workspace`/`-migration` → `workflow_dispatch`-only AND still `disabled_manually` (belt + braces; the auto-on-`junior/*`-push trigger was May's minute burn). Removed dead `phase-v1-SL-a/SL-b` push refs from `adr-compliance`. `adr-drift`/`oq-sweep`/`plan-drift` already dispatch-only since the April private flip — left as-is.
- **Locked posture:** local (native cargo + cargo-linux.sh + Docker e2e) does all compile/test; CR + Copilot review every PR; **Shape G is residual-only** (public green-check for pilot/external PRs). Fire via `gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref <branch>`.

### Theme 2 — Copilot scoped to residual lane (DONE)
- `.github/copilot-instructions.md` created (`d2a33d39a`). Scopes Copilot to what local + CR DON'T cover: whole-PR cross-file consistency, PR title↔diff honesty, test ADEQUACY (not presence), plain-logic bugs in NON-governance code. Explicit "do NOT" list hands compile/type/Diesel/ADR-findings back to local + CR.

### Theme 3 — Diff-vs-trunk lesson (DONE)
- `feedback_verify_branch_diff_vs_trunk_before_concluding_code_missing.md` (`88d71620d`, PMD id 739). Written because I wrongly concluded PR #172 was a "phantom PR with missing code" and nearly re-dispatched impl-2 to re-land e2e tests that were byte-IDENTICAL on trunk. Cross-linked with `feedback_falsifiable_hypothesis_before_structural_fix.md`.

### Theme 4 — Linux-compile gate (DONE this session, NEEDS VALIDATION next session)
The headline deliverable. `cargo-linux.sh` was a tool nothing invoked. Now it's a **diff-scoped bm-pr gate** (`224606866`). Wired at FIVE surfaces:

| Surface | File | What |
|---|---|---|
| DQ kind | `.claude/rules/decision-queue.md` | new `validate-pending-laptop-linux` (sibling of `-e2e`) |
| bm-pr gate | `.claude/commands/bm/bm-pr.md` **Phase-1d** | diff-scope detection + STOP-if-no-passing-DQ |
| advisor stage-shape + handler | `.claude/rules/advisor-orchestrator.md` §3.1 + §5.2 | advisor raises + runs the DQ pre-bm-pr |
| impl-task brief template | `.claude/PRPs/templates/impl-task-brief.template.md` §5 | impl-task writes the DQ when its files match trigger |
| bm-task brief template | `.claude/PRPs/templates/bm-task-brief.template.md` | bm-pr row points at Phase-1d |
| (+ self-doc) | `scripts/brehon/cargo-linux.sh` header | points at the gate |

**Scope (Option-2, user-chosen):** gate fires ONLY when diff touches `Cargo.toml`/`Cargo.lock`/`migrations/**` OR adds `cfg(unix)`/`cfg(target_os)`/path-sep code. Pure governance-logic PRs skip it (Win-green == Linux-green).

**Who produces vs checks:** lane session runs `cargo-linux.sh` → advisor-laptop handler mutates DQ to `pass` → bm-pr (Haiku) only CHECKS the passing DQ. bm-task never runs cargo/Docker.

---

## What was validated this session (don't redo)
- `cargo-linux.sh --version` → exit 0, Linux container works.
- Phase-1d bash extracted + `bash -n` → SYNTAX OK (36 lines).
- Phase-1d python DQ-check block → runs against live DQ, returns `missing` (correct — no linux DQ exists yet).
- Phase-1d diff-scope detection dry-run against redaction-r1 → `LINUX_GATE=0` (correct — its diff is comments + scrub_json cap, no dep/migration/cfg).
- Phase headings in order: 1 → 1b → 1c → 1d → 2. (1c = e2e gate is cited by name in 5+ past briefs — was NOT renumbered; my section is correctly 1d.)

## REMAINING — validate in the new session

1. **End-to-end dry-run of the Linux gate on an IN-SCOPE diff.** All my dry-runs were on out-of-scope/empty cases. Construct or find a branch whose diff touches `Cargo.toml` or adds `cfg(unix)`, run the Phase-1d bash, confirm it STOPs with the right message when no passing DQ exists, and clears when a `validate-pending-laptop-linux` DQ at `result:pass` is present. **This is the one path not yet exercised.**
2. **Write a `validate-pending-laptop-linux` DQ once for real** (next time an in-scope impl-task ships) and confirm the advisor §5.2 handler runs `cargo-linux.sh` + mutates it correctly. The handler prose was added but never executed.
3. **Confirm the new lesson synced to PMD.** Background sync (`bnegx294z`) hit a WARN on an UNRELATED file (`feedback_brehon_subagent_model_effort_assignments.md` — bad frontmatter, pre-existing). Verify `feedback_linux_compile_proof_is_a_gate.md` is searchable via `memory_search_hybrid(query: "linux compile gate cargo-linux", tags: "lesson")`. If absent, re-run `bash scripts/sync-lessons-to-pmd.sh`. (Also: that pre-existing frontmatter-parse WARN is worth a separate fix — the file is skipped by every sync.)
4. **Optional polish:** the Phase-1d cfg-detection grep is a heuristic on diff body (`^\+.*cfg\(unix...`). It won't catch every OS-divergence shape (e.g. a platform-conditional dep in Cargo.toml is caught by the path check, but a `#[cfg(...)]` already-present line being relied on by new code is not). Conservative-by-default mitigates (lane can raise the DQ anyway), but worth a second look if a Linux break ever slips through.

---

## Cross-session state at stop time

- **CONCURRENT BM SESSION (other window):** `branch-manager` running **`bm-triage` on PR #172** (Opus 4.8). It OWNS `.claude/PRPs/reviews/pr-172-findings.yaml` + `.claude/runlog/bm-runlog.md` — both show ` M` in my working tree (its in-progress edits). **DO NOT touch those files.** My commits excluded them (verified pre-commit). PR #172 triage: 5 CR findings, all `fix-in-pr`, 0 critical, recommendation `request-changes`. That session brings triage to user gate 3 + merge to gate 5 — NOT this session's job.
- **PR #172** = the r3c bm-pr. Net diff is meta-only (3 files) because its e2e code already landed on trunk via merge-forward (this is fine — see the diff-vs-trunk lesson). It is NOT a phantom; do not re-dispatch impl-2.
- **redaction-r1** (`brehon-fork-redaction-r1` / `phase-v1-redaction-r1`, Mode A): post-merge-forward at `fd01f6ef9`, stage "e2e re-run required before bm-pr." Pending DQ `9f1d7e7ca817-001` (`validate-pending-laptop-e2e`) at `result: fail` — a stale-branch false-negative; needs e2e re-run → flip to pass → bm-pr. Runs in the redaction-r1 LANE session, not canonical. Its diff is NOT Linux-gate-scoped (verified). Full detail: `.claude/PRPs/handovers/v1-redaction-r1-resume-2026-06-01.md`.
- **governance-v0 tip at stop:** `224606866` (mine). Origin in sync (`0 0`).
- **DQ pending on governance-v0:** 0 (clean, before BM session's in-flight writes).

## Memory updated this session
- `project_shape_g_suspended_2026_05_16.md` — residual-only posture + billing reset.
- `project_laptop_canonical_cargo_runner.md` — cargo-linux.sh DONE (not TODO).
- `feedback_four_tool_review_split.md` (new) — the 5-tool lane split.
- `MEMORY.md` index — Shape-G line + four-tool-split line updated.
- 2 new lessons: diff-vs-trunk (id 739), linux-compile-gate (verify synced).

## Recurring footgun I hit (worth a glance)
- `git show <branch>:<path>` mangled the path twice on Windows (MSYS rewrites `:` + backslashes). The rule says use `scripts/brehon/git-show-json.sh`. I worked around via the handover file instead. Low harm but I kept forgetting the helper exists.
