# Session retro — 2026-05-18 — pmd-stranding-remediation

**Harness:** claude-code
**Session window:** ~2026-05-18 (post-compaction segment of the v1-ship-1-r2 close session) → ~2026-05-18 (~50 min wall-clock for this segment)
**Branch at start:** `81eb9fff2` (`governance-v0`)
**Branch at end:** `53b1a52c1` (`governance-v0`)
**Files touched:** 1 git-tracked (`scripts/sync-lessons-to-pmd.sh`) + the canonical PMD `memory.db` (DB write, not git-tracked: 5 rows + 5 vectors)
**Commits:** 1 (auto: 0, explicit: 1 — `53b1a52c1`)

> Scope note: this retro covers ONLY the post-compaction thread (the PMD-stranding incident discovery → remediation → script structural fix). The pre-compaction thread (the v1-ship-1-r2 multi-cycle L14 recovery + sub-phase close) was already retro'd in `session-retro-2026-05-18-v1-ship-1-r2-l14-recovery-close.md` and the sub-phase retro `v1-ship-1-r2-retro.md`. Per the skill's hard refusal this is a NEW file (distinct thread/slug), not a `-v2` of the existing one.

## TL;DR

A new lesson (`feedback_pmd_cross_lane_canonical_db.md`) was authored this phase warning that `sync-lessons-to-pmd.sh` strands lessons in a lane-local DB when run from a lane worktree — and then that exact bug stranded all 5 of this session's lessons (including, with perfect irony, that very lesson) in `brehon-fork-ship-1/.project-memory/memory.db` instead of the canonical `brehon-fork/.project-memory/memory.db`. The lesson described the failure mode but the *script itself was never fixed* — knowing a footgun exists is not the same as removing it. The session diagnosed it (5-lesson blast radius, search-index-only impact, lesson `.md` files safe on origin), remediated it (synced 5 to canonical via the new `--db` flag, backfilled embeddings → canonical 317/317/0-missing), and shipped the structural fix (`53b1a52c1`: DB-resolution precedence + a lane-local sanity-check WARN + an arg-parse bug fix). Top change proposal: a lesson that documents a footgun in a *fixable script* must be paired with the script fix in the same or immediately-following commit — a documented-but-unfixed footgun re-fires on the next run and is strictly worse than no lesson (it gives false confidence the risk is "handled").

---

## What surprised us

- **The just-authored lesson was itself stranded by the bug it documented.** `feedback_pmd_cross_lane_canonical_db.md` was written this phase to warn about exactly this stranding. It was then synced — by the unfixed script, from the lane worktree — into the lane-local DB, making the lesson about cross-lane stranding invisible to cross-lane search. The lesson's own existence depended on the fix it implied but did not perform. This is a sharp instance of "a lesson is necessary but not sufficient; the structural fix is what removes the risk."
- **The first verification query used the wrong (lane-local) DB path** and would have falsely reported "5 lessons present, all good" — the same lane-local-default reflex that caused the bug almost masked it during verification. The catch was switching the verify path explicitly to the canonical absolute path. Verification tooling inherits the same default-path footgun as the thing it verifies.
- **The blast radius was exactly the clean expected set (5 of 134), 129 already in canonical.** No partial/dirty divergence, no older stranded rows from prior lane sessions — the canonical DB was otherwise complete. That made remediation a clean idempotent re-sync rather than a reconciliation. Pleasant surprise; it could easily have been a messy multi-session divergence (cf. the separate stranded-`Task retro:` reconciliation whose debug scaffolding — `.claude/PRPs/debug/reconcile-*.json` — is still on disk untracked, and which was a *messier* manifestation of the same root class earlier this phase).
- **The script's `--db <path>` space-form silently did nothing before the fix.** The original arg-parse was `for arg in "$@"` with `--db) shift; DB=...` inside it — `shift` does not affect a `for` loop's iteration, so `--db <path>` parsed `--db` as unknown-arg-exit OR ignored the value while `--db=<path>` worked. A latent parsing bug sitting *inside the very script that needed an override flag to be safe* — the safety mechanism was half-broken before it was ever exercised.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When authoring a lesson that documents a footgun in a **fixable artifact under our control** (a script, a wrapper, a config the repo owns), pair the lesson with the structural fix in the same commit, OR open the fix as the explicit next action and note it in the lesson body as "fix status: SHIPPED `<sha>` / PENDING". A lesson documenting an unfixed-but-fixable footgun gives false confidence. Promote as `.claude/lessons/feedback_lesson_must_pair_with_structural_fix_when_fixable.md` (proposed). | Removes the "documented but still bites" failure class. The next sync from a lane can no longer silently strand. | minor (one lesson + the discipline) | 2× this phase (stranded-`Task retro:` reconciliation AND this stranded-lesson-sync — same root: lane-local-default writes), 0× prior as a *named* discipline |
| 2 | Make the Step 5.5 backfill block in `.claude/skills/session-retro/SKILL.md` and `post-task-retro` use an **explicit canonical `PROJECT_MEMORY_DB`** (`C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`) and an explicit canonical sync (`bash scripts/sync-lessons-to-pmd.sh --db <canonical>`), NOT the relative `.project-memory/memory.db` the template currently shows. The template's relative path is the exact lane-local-default that strands when the retro runs from a lane worktree. | The retro/backfill discipline stops *causing* the stranding it is meant to close. Self-consistent tooling. | minor (edit the Step 5.5 code block in two skill files) | 2× this phase (the stranding happened *during* the prior session-retro's Step 5.5 — the skill block itself was the vector) |
| 3 | Add a one-line guard to `scripts/sync-lessons-to-pmd.sh` exit summary OR the retro Step 5.5 verify: after sync, assert the resolved `$DB` is NOT under a `brehon-fork-*/.project-memory/` path and FAIL (exit non-zero) rather than WARN when it is, when invoked with `--strict` (default WARN preserved for back-compat). The current WARN is correct but non-fatal — a future automated retro could ignore stderr and strand silently again. | A WARN can be missed by an automated caller; a `--strict` fail cannot. Defence-in-depth on top of the precedence fix already shipped in `53b1a52c1`. | minor (one `case` arm + a flag) | 1× this phase + the general "WARN gets ignored by automation" pattern (cf. the PMD silent-FTS5-degrade lesson `feedback_pmd_backfill_after_write.md`) |

## What to carry forward

- **Trust-but-verify the *verification path itself*, not just the action.** The verify query nearly inherited the same lane-local-default that caused the bug. When remediating a path/target-resolution bug, the verification must hard-code the *correct* target explicitly and assert against it — never reuse the resolution logic under test. Used once cleanly here (switched verify to the canonical absolute path and confirmed ids 404–408 + 0-missing-vectors).
- **Blast-radius-first before remediation.** The session did a direct title-diff (canonical vs lane-local) to establish "exactly 5 missing, 129 present, clean expected set" *before* running the real sync. That converted an unknown-scope incident into a known idempotent re-sync, and let the remediation be a single confident `--db` invocation rather than a cautious reconciliation. Cheap (one Python title-diff), high signal — make it routine for any "data landed in the wrong place" incident.
- **Idempotent sync + idempotent backfill make remediation low-risk.** `sync-lessons-to-pmd.sh` skips-on-title-exists and `backfill.js` is idempotent on `(memory_id, model)`. Because both are idempotent, the remediation could not double-insert or re-embed — the only question was "does it reach the right DB", which the blast-radius diff had already answered. Carry forward: prefer idempotent sync/backfill tooling so remediation is "re-run pointed at the right place" not "surgically insert the missing N".
- **Respect the canonical-checkout hard refusal even under remediation pressure.** The canonical `brehon-fork` checkout was detached-HEAD with the federation-inbound-a lane's debris and in active use as that lane's cargo/e2e runner. The remediation deliberately read *this lane's* git-tracked lesson files and wrote *only the canonical DB*, never `git checkout`/`pull`-ing the canonical working tree (`multi-lane-worktree.md` hard refusal). The DB write and the working tree are independent — exploit that separation rather than disturbing a live lane.
- **Never edit the Stop hook to fix a stranded-retro symptom.** Per the rule, the hook's git-common-dir → canonical resolution is *correct*; the defect is always MCP-side / sync-side (a lane-local target). Held this line — fixed the offending script's pointer, not the hook.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Blast-radius title-diff (canonical vs lane-local, ad-hoc Python) | 20 | 0 | medium | Converted unknown-scope incident → known idempotent re-sync; the "exactly 5, clean set" result was a relief vs a feared messy divergence |
| `sync-lessons-to-pmd.sh --db <canonical>` (the new flag, dogfooded) | 15 | 3 | low | 3 min wasted on the `--db` space-form parse bug discovered mid-remediation → folded into the structural fix; net strongly positive (idempotent, 5 imported / 129 skipped / 0 errors) |
| `backfill.js` against canonical (Ollama homeserver:11434) | 10 | 0 | none | `done=5 failed=0` in 23.8s; Ollama precondition check passed first try |
| First verify query (lane-local path — wrong) | 0 | 4 | medium | Almost masked the bug by querying the same wrong DB; caught by switching to explicit canonical path. The waste IS the carry-forward lesson |
| Script structural fix `53b1a52c1` (precedence + WARN + arg-parse loop) | — | — | low | 1 file / 65+13 / `bash -n` clean; removes the recurring class. complexity below |
| /session-retro (this invocation) | — | — | none | correctly scoped to the post-compaction thread; did not re-litigate the already-retro'd L14 recovery |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. Format: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| PMD-stranding remediation (diagnose + sync + backfill + verify) | 1 (DB) | 0 | ~25 | ~0 (interactive, no long bg silence — `by6xzpkar` bg sync was the only wait, ~few min) |
| `sync-lessons-to-pmd.sh` structural fix `53b1a52c1` | 1 | 1 | ~15 | ~0 |

No task exceeded the >55min / >40min-silence / >8-files thresholds. This was a small, well-scoped remediation segment — no watchdog/bundling concern.

## Decisions to revisit

- The `.claude/PRPs/debug/reconcile-*.json` + `reconcile-payloads/` + `v1-ship-1-stranded-retros-staging.json` artifacts are untracked debug scaffolding from the *earlier* stranded-`Task retro:` reconciliation (a sibling manifestation of the same lane-local root class). Decide: gitignore the `.claude/PRPs/debug/` pattern, or clean these up now that v1-ship-1-r2 is closed. One line; not urgent. Surfacing because they are evidence the root class manifested **twice** this phase (reinforces "What to change" #1's recurrence count).
- Should `.mcp.json.example`'s `_comment_pmd_cross_lane` guard be promoted into a committed pre-commit or session-start check that asserts the lane's `.mcp.json` `PROJECT_MEMORY_DB` is the canonical absolute path? The lesson + the script fix close the *sync* path; the *MCP-write* path (memory_write_eval / memory_write) still relies on each lane's `.mcp.json` being correct by convention. Worth a clarify before the next multi-lane phase.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

Boxes UNCHECKED by default; user checks to authorise.

- [ ] **"What to change" #1**: promote to `.claude/lessons/feedback_lesson_must_pair_with_structural_fix_when_fixable.md` (a lesson documenting a footgun in a repo-owned fixable artifact must ship/track the structural fix, not just describe the risk). Recurrence: 2× this phase (stranded-`Task retro:` reconciliation + stranded-lesson-sync). Cross-references `feedback_pmd_cross_lane_canonical_db.md`, `feedback_principles_not_rules.md`.
- [ ] **"What to change" #2**: update `.claude/skills/session-retro/SKILL.md` Step 5.5 + `.claude/skills/post-task-retro/SKILL.md` to use explicit canonical `PROJECT_MEMORY_DB` / `sync-lessons-to-pmd.sh --db <canonical>` instead of the relative `.project-memory/memory.db` (the relative path is the lane-local-default that strands when the retro runs from a lane worktree — it was the actual vector this phase).
- [ ] **"What to change" #3**: add `--strict` (fail-not-WARN on lane-local DB) to `scripts/sync-lessons-to-pmd.sh` and call it with `--strict` from the retro Step 5.5 (defence-in-depth; a WARN can be ignored by an automated caller).
- [ ] **Decisions-to-revisit (mcp guard)**: PMD eval write + a clarify on promoting the `.mcp.json` canonical-path guard into an enforced session-start check (requires `PROJECT_MEMORY_DB` exported; canonical path).

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Scope: post-compaction PMD-stranding
remediation thread only; pre-compaction L14 recovery retro'd separately in
`session-retro-2026-05-18-v1-ship-1-r2-l14-recovery-close.md`._
