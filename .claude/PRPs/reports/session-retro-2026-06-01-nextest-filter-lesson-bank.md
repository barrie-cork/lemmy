# Session retro — 2026-06-01 — nextest-filter-lesson-bank

**Harness:** claude-code
**Session window:** ~2026-06-01T22:00Z → 2026-06-01T22:45Z (~45 min, mostly e2e compile wait)
**Branch at start:** `claude/refine-local-plan-aqJqJ` (PR #176 head) + `governance-v0` for the lesson
**Branch at end:** `775219766` (`governance-v0`)
**Files touched:** 1 committed (the lesson) + 1 pre-existing uncommitted not-mine (`docker/lemmy.hjson`)
**Commits:** 1 (auto: 0, explicit: 1)

## TL;DR

Tail end of the ADR-017 / BUG-1 thread (PR #176). The three new author-defendant e2e tests
passed locally (**3/3** via the `E2E_EXIT_0` sentinel) — the runtime proof the cloud/Ultraplan
env couldn't produce (no Docker for testcontainers). The user approved banking the Windows-nextest
tooling lesson observed during the run; it was authored, committed, pushed to `governance-v0`, and
verified searchable in the live PMD. The session's dominant signal — and the top carry-forward — is
that **four separate harness/script exit-summaries disagreed with captured ground truth** in one
short session; every one was caught only by reading the actual artifact (sentinel, SQLite row, live
MCP query). `pattern_verify_before_trusting_shell_output.md` is already a promoted 3+ pattern; this
session is strong fresh evidence, not a new lesson.

---

## What surprised us

- **Four false exit-summaries in 45 minutes.** (1) The first e2e re-run's harness task-notification
  said "exit code 0" while the log's appended sentinel said `E2E_EXIT_NONZERO` (the filterset parse
  failure). (2) The successful e2e run's harness summary said exit 0 — this time *correct*, but only
  confirmable against the `E2E_EXIT_0` sentinel. (3) `sync-lessons-to-pmd.sh` printed
  `imported: 0   skipped: 188` for a corpus that had just gained a genuinely new file — yet the row
  (id 762) was demonstrably inserted and FTS5-searchable. (4) That same sync exited `2` (its own
  footer `===SYNC EXIT===2`) while the harness task-notification reported "exit code 0". The rate of
  summary-vs-truth divergence in a single short session was higher than expected.
- **PMD topology turned out NOT to be split for this write.** `pmd-invariants.md` #1 says the
  `PROJECT_MEMORY_DB`/SQLite-file mechanism is "superseded" by the HTTP daemon — I expected the
  script's raw-`sqlite3` write to strand the row in a store the live `memory_search_hybrid` never
  reads. It did not: the live MCP query returned the row as the #1 hit. In this session the SQLite
  file and the HTTP-daemon store were the same backing DB. (Worth not over-generalising — the script
  *should* still write via the MCP to be topology-proof, see §What-to-change.)
- **`governance-v0` was 14 commits ahead of origin** at lesson-commit time — a whole tail of prior
  smoke-test/retro meta-work (Phases 8–9, retros, V2 OQs) that had never been pushed. A one-lesson
  push grant would have silently shipped all 15. Surfaced + got explicit approval before pushing.
- **The "lessons auto-sync" hook isn't wired here.** The session-retro skill body (Step 5.5) states
  lesson→PMD sync "is now automated via `.claude/hooks/lesson-pmd-sync.sh`." But in this canonical
  checkout that hook is present-but-unwired in `settings.local.json` — it did not fire; the manual
  `sync-lessons-to-pmd.sh` did the indexing. The skill's stated automation is aspirational here, not
  actual — a `settings.local.json` non-propagation gap (`feedback_settings_local_json_worktree_bootstrap.md`).
  **Fixed 2026-06-02:** `PostToolUse` (matcher `Edit|Write`) now wired → `lesson-pmd-sync.sh` (JSON validated).
- **My own retro carried an unfalsified hypothesis (caught at execution time).** §What-to-change #1
  (the sync counter "bug") was logged from the opaque `imported: 0` summary WITHOUT a falsification
  pass. On execution (2026-06-02) I falsified it in ~5 min — grep showed only the safe `$((x+1))`
  form, an isolated bash test showed that form doesn't trip `set -e`, and a full INSERT+increment
  simulation returned `imported=1`. There was no bug. The irony is sharp: I committed the exact
  `feedback_falsifiable_hypothesis_before_structural_fix.md` failure mode *inside the retro that was
  cataloguing summary-distrust*. The discipline must extend to my OWN proposals, not just to harness
  output — a retro's structural-fix items are hypotheses too, and must carry a falsification before
  they're acted on.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | ~~Fix the `imported`/`skipped` counter in `sync-lessons-to-pmd.sh`~~ **WITHDRAWN 2026-06-02 — misdiagnosis.** Falsified during execution: the script uses the SAFE `imported=$((imported + 1))` form throughout (no `((x++))`, no `let` — grep-confirmed), which does NOT trip `set -e` (isolated bash test confirmed `$((x+1))` survives, only `((x++))` aborts on a 0 result). The full INSERT-then-increment sequence was simulated under `set -euo pipefail` → `imported=1`, exit 0. The original `imported: 0` reading was a **misread of an opaque summary against an already-populated DB** (the row id 762 had inserted; the dry-run correctly reports "0 would-import, 189 already in DB"). There is no bug. Lesson: I logged a structural-fix proposal from an opaque summary WITHOUT falsifying it — the exact `feedback_falsifiable_hypothesis_before_structural_fix.md` failure mode, committed in the very retro cataloguing summary-distrust. | — (no change — there was no defect) | — | misdiagnosis, withdrawn |
| 2 | Make `sync-lessons-to-pmd.sh` write via the `memory_write` MCP tool (HTTP daemon) instead of raw `sqlite3 INSERT`, per `pmd-invariants.md` #1. The session-retro skill body already says the SQLite path is "superseded" and lessons MUST go via `memory_write` — but this standalone script still does raw SQLite. The two contradict. | Eliminates the latent strand-risk the moment the HTTP daemon and SQLite file diverge (they happened to coincide this session; that's luck, not guarantee) | medium | 1× here + standing contradiction with the documented topology (≥1 prior in `feedback_pmd_retro_check_http_store_split.md`) |
| 3 | When running a backgrounded build/test whose truth is a `&& echo SENTINEL` tail, **always read the sentinel, never the harness exit-summary** — and state that explicitly in the closing report. (Already my practice; codify as the default so it survives compaction.) | No false-green from a piped/wrapped exit code — the exact failure class `cargo-output-capture.md` + `feedback_task_notification_exit_summary_unreliable.md` warn about | minor (habit) | 4× this session (the four summaries above); ≥3× in prior memory (promoted pattern) |

## What to carry forward

- **Sentinel-append discipline on every backgrounded cargo/test run** (`&& echo E2E_EXIT_0 || echo E2E_EXIT_NONZERO`). It caught the first run's filterset failure in ~45s *before* any compile, and was the sole reliable signal three more times this session. This is the single highest-value habit in the session.
- **Surface branch divergence before a scoped push.** The "14 ahead / one-lesson grant" check took ~10s (`git rev-list --left-right --count`) and prevented silently shipping 14 commits under a narrow grant. Verified none touched `crates`/`migrations`/`tests` before asking. Keep this as a pre-push reflex whenever a push grant is narrower than the branch's unpushed delta.
- **Verify-before-trusting at every layer, including the "it's indexed" claim.** Didn't stop at "the script ran" — queried SQLite directly (row present), then the live MCP (searchable, #1 hit). "Wrote" ≠ "searchable"; only the live `memory_search_hybrid` proves the latter.
- **Honest provenance in the artifact, not just the chat.** The user flagged they couldn't observe the Windows mangling directly (Linux). The lesson file carries a `Provenance (2026-06-01)` block scoping the claim to win32 / authored-from-report. Sourcing honesty belongs in the durable artifact.
- **Respect phase-branch.md meta-vs-code split under pressure.** Lesson = meta → direct on `governance-v0`, not folded into code PR #176, even though folding would have been one fewer push. The two-track discipline held.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| e2e sentinel-tailed bg run (`cargo-nextest.bat`) | 25 | 1 | medium | Sentinel caught the first filterset failure pre-compile (~45s, ~1 min wasted); the positional-filter fix held on re-run; 3/3 green. The "saved" is vs a false-green that would have shipped an unverified PR. |
| `memory_search_hybrid` (verify index) | 5 | 0 | medium | Confirmed lesson searchable (#1 hit) — the only test that actually proves indexing. Surprise: disproved the expected HTTP/SQLite strand. |
| `sync-lessons-to-pmd.sh` | 3 | 6 | high | Did insert the row, but its `imported: 0` + exit-2 summary forced a 3-probe cross-check (SQLite direct + live MCP) to trust. Net-negative on friction this run; §What-to-change #1+#2. |
| `AskUserQuestion` (land-method + 14-ahead) | 8 | 0 | none | Two clean gates: land-lesson method, then the 14-ahead push approval. Prevented an unintended 14-commit push. |
| `git rev-list --left-right` divergence check | 10 | 2 | medium | The ~2 min "wasted" was me first *misreading* the left/right direction (thought 14-behind), corrected by listing both sides concretely. Cheap, and the correction itself is a carry-forward. |
| `memory_write_eval` (#763) | — | — | none | Stop-hook-mandated retro; clean. |

## Complexity scores (heavy tasks only)

No heavy multi-file tasks this session (the BUG-1 impl ran in the cloud/Ultraplan env as PR #176, not here). The only local artifact was a single-file lesson commit.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Lesson author + land + PMD-verify | 1 | 1 | ~20 | n/a (interactive) |
| e2e verify (bg) | 0 (read-only) | 0 | ~10 (8m32s compile + 60s run) | ~8m (single compile-silence window, expected) |

Neither flags a watchdog/envelope concern.

## Decisions to revisit

- The standing contradiction between session-retro Step 5.5 ("lessons MUST go via `memory_write`; SQLite path superseded") and `sync-lessons-to-pmd.sh` (still raw `sqlite3`). **Resolved structurally 2026-06-02 by wiring the `lesson-pmd-sync.sh` PostToolUse hook** (the MCP-write path) — the standalone SQLite script is now a fallback, not the primary path. The script-via-MCP rewrite is no longer urgent; the hook supersedes it for interactive sessions.
- PR #176 CodeRabbit four-bucket triage is the one open downstream item — user's call to trigger; not a retro decision.

---

## Execution log (2026-06-02 — user asked to implement all three §What-to-change items)

- **#1 (sync counter fix) — WITHDRAWN, no defect.** Falsified the hypothesis before editing: grep showed only the `set -e`-safe `$((x+1))` form; isolated bash test + full INSERT-then-increment simulation both confirmed correct counting. The original `imported: 0` was a misread against an already-populated DB, not a bug. No change made. (Meta-lesson recorded in §What surprised us.)
- **#2 (wire `lesson-pmd-sync.sh`) — DONE.** Added `PostToolUse` (matcher `Edit|Write`) → `bash .claude/hooks/lesson-pmd-sync.sh` to `.claude/settings.local.json`; JSON validated; `.mcp.json` confirmed to carry `project-memory.url` (HTTP topology) so the hook's precondition holds. Takes effect next session (hooks load at SessionStart). This is the real fix that supersedes both #1's phantom and the script-via-MCP item.
- **#3 (sentinel-over-summary) — DISCIPLINE, no code.** Already covered by promoted patterns; proved its worth again this very leg (distrust of `imported: 0` is what falsified #1). Nothing to implement.

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [x] §What-to-change #2 (wire lesson-pmd-sync hook): **executed** — `.claude/settings.local.json` PostToolUse added.
- [ ] §What surprised: "my own retro item was an unfalsified hypothesis" — recurrence is 1× here; if a 2nd retro-self-misdiagnosis appears, promote `feedback_falsify_own_retro_proposals.md` (extend `feedback_falsifiable_hypothesis_before_structural_fix.md` to cover the author's own retro items). Single-instance for now — recorded, not promoted.
- [ ] §What-to-change #3 (sentinel-over-summary): already covered by `pattern_verify_before_trusting_shell_output.md` + `feedback_task_notification_exit_summary_unreliable.md` + `cargo-output-capture.md` — no new lesson; corroborating evidence only.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase section omitted — no `/auto-phase`
interaction this session (leftover `v1-RT-r3.json` is a stale prior-session artifact)._
