# v1 Close-Out — Isolated-Lane Setup Report

> **Purpose:** capture what actually happened bringing the
> `brehon-fork-closeout` lane online + harness-verifying it, so future
> isolated-lane setups for Brehon work are faster and avoid the gaps found
> here. **Forward-looking** — complements (does not duplicate) the
> as-designed `v1-closeout-bootstrap.md` handover and the canonical
> session's `session-retro-2026-06-04` (planning side).
>
> **Authored:** 2026-06-04 by the **execution** session (CWD
> `brehon-fork-closeout`, branch `phase-v1-closeout`). The lane itself was
> *created* by the canonical advisor session; this report documents the
> *first-session harness verification* of it.
>
> **Scope of this session:** verify the harness is fully working before any
> plan execution (user directive: "first ensure the harness system is fully
> working" + "finish the harness check"). No close-out *tasks* were
> executed; this is pure setup-verification + two fixes.

---

## TL;DR — lane is GREEN, two gaps found & fixed, three hygiene items logged

| Layer | Status | Notes |
|---|---|---|
| Worktree isolation | ✅ | Dedicated `brehon-fork-closeout` on `phase-v1-closeout`; M1 (`brehon-fork-validate`/`phase-m1-b`) untouched. |
| Bootstrap artifacts (4-step) | ✅ | `.mcp.json`, `.env`, `settings.local.json` present; submodule initialized. |
| SessionStart/Stop/Pre/PostToolUse hooks | ✅ | All wired (2 SessionStart + Stop + PreToolUse + PostToolUse). |
| PMD (HTTP daemon) | ✅ | `localhost:11435` live; hybrid search (semantic+FTS5) round-trips; ollama reachable. |
| Junior daemon | ✅ read-only | PID 2120848, 136h up, 0 jobs. **Reserved for M1 — close-out does NOT dispatch to it** (user directive). |
| Cargo wrapper (4 probes) | ✅ 1,2 / ⏳ 3,4 | Probe 1 (`-p`) + Probe 2 (`--features full`) PASS; Probe 3 (e2e compile) + Probe 4 (negative) in flight. |
| **GAP 1 — Telegram completion hook** | ✅ FIXED | Hook was broken (`BuildMessage: ModuleNotFound`); removed + recreated (now ID 2, clean). |
| **GAP 2 — missing lesson file** | ✅ FIXED | `feedback_daemon_telegram_completion_hook.md` was cited by an always-load rule but didn't exist; reconstructed. |
| Hygiene items (for Phase 1) | 📋 logged | DQ bloat (545 KB / 273 resolved), 55 active plans, 8 worktree dirs on disk. |

---

## What the bootstrap got RIGHT (keep doing this)

The canonical session's bootstrap was high quality. These worked first-try:

1. **Submodule init in the worktree.** `crates/email/translations` was
   populated (8 entries) — so `lemmy_email`'s `build.rs` did NOT fail with
   `Os { code: 3, kind: NotFound }`. This is the #1 lane-bootstrap footgun
   (`feedback_phase_lane_worktree_bootstrap_checklist.md`) and it was
   handled.
2. **`.mcp.json` under the HTTP-PMD topology.** No `PROJECT_MEMORY_DB`
   env-var (correct post-2026-05-30 — the HTTP daemon manages the DB
   server-side). PMD hybrid search returned relevant v1 retros on the first
   query.
3. **Both SessionStart hooks wired** (`pmd-canonical-guard.sh` +
   `session-start-multi-lane-check.sh`) — the multi-lane drift detector is
   live, which matters precisely because M1 is running concurrently.
4. **A real cold-start bootstrap handover** (`v1-closeout-bootstrap.md`,
   119 lines) — self-contained, with the M1-merge gate commands, the
   approval decisions, and the carry-patch audit pre-baked. This is the
   gold standard for lane handoffs.

---

## GAP 1 — Telegram completion hook was silently broken

**Symptom:** `mcp__junior-brehon__list_hooks` showed hook ID 1 `active` but
with `LAST ERROR: BuildMessage: ModuleNotFound...` and `LAST TRIGGERED:
never`. The ✅/❌ task-completion notifier would not have fired.

**Root cause:** the daemon was restarted on May 29 (136h uptime at check
time). Across a daemon restart/version change, the hook's stored `check_fn`
referenced a module the current daemon runtime could no longer resolve.
This is the exact failure mode `advisor-orchestrator.md` §1 "Telegram
completion hook check" warns about ("daemon restarts wipe hooks").

**Fix applied:** `remove_hook(1)` → `create_hook(<natural-language ✅/❌
description>)`. The daemon recompiled the check_fn against the live runtime;
new hook is **ID 2** (`junior-task-terminal-notify`), `active`, no error.

**Improvement for future lanes:**
- The session-start hook-check should compare on **error-presence**, not
  just **absence**. The rule said "recreate if absent" — but the failure
  here was *present-but-broken*. → I updated the reconstructed lesson with a
  3-state decision table (OK / absent / present-with-error).
- **Hook IDs are not stable across recreation.** Rules/handovers that say
  "hook ID 1 must exist" are wrong post-recreation. Cite the hook by
  **purpose**, not number. (Lesson updated.)
- This hook is **daemon state, not repo state** — it does NOT travel with
  the worktree. Every new lane that wants completion pings must verify it
  against the live daemon, regardless of how clean the git bootstrap was.

---

## GAP 2 — an always-loaded rule cited a lesson file that did not exist

**Symptom:** `advisor-orchestrator.md` §1 (an *always-load* rule) instructs:
"recreate hook ID 1 via `create_hook` with the same payload documented in
`feedback_daemon_telegram_completion_hook.md`." That file **did not exist**
anywhere on disk (verified across all 3 worktrees + a filesystem-wide
`find`). Four handover files also referenced it.

**Root cause:** lesson-corpus drift — the lesson was indexed in MEMORY.md
and cited by rules/handovers, but the actual `.md` was never authored (or
was lost in a prior cleanup). The citation chain
(`pattern_lesson_lifecycle_chain.md`: PMD → lessons/ → rule → handover)
broke at the `lessons/` hop.

**Fix applied:** authored
`.claude/lessons/feedback_daemon_telegram_completion_hook.md` capturing the
recreation procedure, the 3-state decision table, the ID-instability note,
and the daemon-DB-topology caveat (below). The rule's reference now
resolves.

**Improvement for future lanes / general harness health:**
- A periodic **"do all rule/handover lesson-citations resolve to real
  files?"** sweep would have caught this. Candidate for the Phase 1d
  lesson-corpus workflow (it already plans to surface
  "historically-cited-but-never-authored slugs"). This is a concrete
  instance to seed that audit.
- When a lane is bootstrapped, the always-load rule corpus is inherited
  verbatim — a broken citation in it is inherited too. **The bootstrap
  checklist should include a citation-integrity check**, not just
  file-presence checks.

---

## Daemon-topology finding (documented, not a gap)

While diagnosing GAP 1, two things looked alarming but are **benign** —
recording so future sessions don't re-investigate:

1. **3 `junior daemon` processes** on homeserver (PIDs 2120848 / 2978254 /
   2978267). NOT competing instances — they are **separate templated
   systemd units for different repos**: 2120848=`brehon-fork`,
   2978254=`/srv/dog-shelter`, 2978267=`/srv/food-producer` (confirmed via
   `lsof` — each holds only its own repo's `junior.db`). Expected
   multi-repo topology.
2. **The SSH-reachable `~/.junior/junior.db` shows `0 hooks`** while
   `list_hooks` (MCP) correctly returns the live hook. The MCP-connected
   daemon instance and the SSH-queried DB file are **not the same store**.
   → **Trust `list_hooks` (the live MCP), never cross-check hook state
   against an on-disk `junior.db`.** (Recorded in the reconstructed lesson.)

---

## Cargo-wrapper audit (the mandatory pre-phase 4-probe gate)

Per `.claude/rules/pre-phase-harness-audit.md` + the SessionStart reminder.
Even though close-out Phases 0–3 touch zero `crates/`, the wrapper must be
proven green before Phase 4 (deps-r2) relies on it.

| Probe | Command | Expect | Result |
|---|---|---|---|
| 0 | `docker ps` | DOCKER OK | ✅ OK |
| 1 | `cargo-check.bat -p lemmy_utils` | only `lemmy_utils` compiles, exit 0 | ✅ exit 0, 2m19s, scope honored |
| 2 | `cargo-check.bat -p lemmy_db_schema --features full` | full schema compiles, exit 0 | ✅ exit 0, 8m36s, `--features full` honored |
| 3 | `cargo-test.bat --test e2e --no-run -p lemmy_server` | e2e binary compiles | ⏳ in flight (heavy — pulls wasmtime/server) |
| 4 | `…--features nonexistent_xyz` (×2) | **non-zero** exit (propagation) | ⏳ runs after Probe 3 |

Logs: `.claude/audit-cargo-*.log`. On all-4-pass: `touch
.claude/audit-phase-v1-closeout-complete.flag`.

**Improvement for future lanes:** Probes 2+3 together are ~17+ min of cold
compile. For a fresh lane (cold target dir), budget **~20–25 min** for the
wrapper audit alone. Consider running Probe 1 (fast, 2 min) as a
smoke-check first, then Probes 2/3 in series while doing other setup —
which is what this session did (overlapped the audit with hook fix + report
authoring).

---

## Hygiene items the setup surfaced (hand to Phase 1)

These are not setup failures — they're inherited debt the close-out plan
already targets, but quantified here as a baseline:

1. **DQ bloat:** `.claude/decision-queue.json` = **545 KB, 0 pending, 273
   resolved.** Over BOTH archive triggers (>100 entries, >200 KB) from
   `decision-queue.md` archive policy. → run `dq-archive.sh` early in Phase
   1 (the lane inherited the full `governance-v0` DQ history).
2. **55 active plan files** (only 7 in `completed/`) — matches Phase 1b's
   archival target. M1 + close-out plans stay active; the rest → `completed/`.
3. **8 `brehon-fork-*` dirs on disk** (3 live worktrees + ~5 stale) —
   Phase 1a names 6 verified-empty stale dirs for `rm -rf` (inspect-then-
   confirm, user go-ahead required).
4. **MEMORY.md over budget** (was 202/200; this session added 1 HARD-RULE
   line for the no-EliteDesk directive → now 203). Phase 1c milestone-prune
   handles it.
5. **Lessons: 195** (this session +1 for the reconstructed Telegram lesson).

---

## Improvement checklist for the NEXT isolated-lane bootstrap

Distilled — what to add to `feedback_phase_lane_worktree_bootstrap_checklist.md`
based on this session:

- [ ] **Verify the Telegram completion hook by ERROR-STATE, not just presence**
      — `list_hooks` and check both "exists" AND "no `last_error`". Recreate
      on either failure. Cite the hook by purpose, not ID number.
- [ ] **Submodule-init verification must test for a gitlink FILE, not a `.git`
      DIR** — in a worktree the submodule `.git` is a *file* (pointer), so
      `[ -d submodule/.git ]` gives a false negative. Use `git submodule
      status <path>` (no leading `-` = initialized) or count dir entries.
- [ ] **Citation-integrity check** — after inheriting the always-load rule
      corpus, grep rules+handovers for `feedback_*.md` references and confirm
      each resolves to a real file. A broken citation in an always-load rule
      is inherited by every new lane.
- [ ] **Budget ~20–25 min for the cold cargo-wrapper audit** (Probes 2+3 are
      the long pole). Overlap it with non-cargo setup work.
- [ ] **Trust `list_hooks` (live MCP) over on-disk `junior.db`** for hook
      state — they can be different daemon stores.
- [ ] **Quantify inherited DQ size at bootstrap** — a fresh lane inherits the
      trunk's full DQ history; if >200 KB, archive before the first DQ write.

---

## Skill-worth flags (surfaced during execution — Phases 1 + 3, 2026-06-04)

What the close-out *execution* (not just bootstrap) surfaced as worth a skill,
a skill fix, or a new automation. Logged here so the close-out retro (Phase 8)
can promote the durable ones; none is urgent.

### A. Existing skills that earned their keep (no change needed)
- **`memory-prune`** — did exactly its job at the v1→M1 boundary (Phase 1c).
  Its byte-budget framing (bind on bytes, not lines), the transfer-test for
  milestone cuts, and the "archive line is itself a byte cost" warning all
  fired correctly. The skill's own caution that a first pass can land *under
  lines but over bytes* matched reality: pass 1 recovered ~551 B (estimated
  ~920), needed a pass 2. **Keep as-is.**
- **`dq-archive.sh`** (script, not skill) — the keep-cited citation guard is
  the load-bearing safety: it kept 30 cited integer-id entries live that a
  naive cutoff would have orphaned. **Keep as-is.**

### B. Candidate NEW skill — `carry-patch-upstream-audit` (MEDIUM value)
Phase 3 ran as an ad-hoc `Workflow` script (4 verify agents → 2 PR-draft
agents). The shape is **repeatable every upstream rebase**: enumerate
`TODO(brehon-fork)` markers → classify governance-free vs fork-local →
diff each candidate against `upstream/main` → draft PRs. Worth promoting to a
skill IF carry-patch upstreaming becomes a recurring cadence (it will, per
`feedback_carry_patch_todos.md`). Captured findings the skill should encode:
- The fork's working-tree diffs are **NOT cherry-pickable** — they bundle
  unrelated edits + governance-coupled fixture args. A skill must isolate the
  per-site hunk, never `git diff`-port.
- An audit's highest value is **refuting the launcher's own hypotheses** (the
  toolchain-divergence premise was disproven; risk relocated to a different
  unverifiable fact). Frame verify-agents to falsify, not confirm.
- **Defer** authoring until the 2nd upstream-rebase carry-patch sweep (1×
  recurrence so far — promote on the 2nd per the lesson-lifecycle bar).

### C. Skill GAP — no skill detects/repairs broken always-load citations
GAP-2 (a lesson cited by an always-load rule but absent on disk) was found by
a hand-run grep sweep, fixed by hand (2 lessons reconstructed in Phase 1d).
The detector is a one-liner (`grep rule citations | test -f each in lessons/`)
and the `new-lane` skill's Step 7 already runs it at bootstrap — but there is
**no skill that re-runs it on demand or repairs the gap**. Options:
- Cheapest: add the Step-7 sweep to `memory-prune`'s Step 3.5 (it already
  audits rule-side integrity) as a citation-resolves check.
- Or a tiny standalone `citation-integrity` skill (sweep → list broken →
  offer to scaffold a stub lesson from the citing rule's inline text).
- **Recommend** folding into an existing skill, not a new one — the detector
  is 1 line; the value is the *reconstruct-from-inline-rule-text* step, which
  is judgment, not automation.

### D. Cross-cutting automation flag (NOT a skill — a guard)
Two exit-code-capture traps recurred this session (`| tail` and
`& echo %errorlevel%` both masked a real `101`). Both are already documented
lessons (`feedback_pipes_mask_exit_codes.md`,
`feedback_cmd_c_redirect_exit_code_capture.md`) — so this is a **discipline
miss, not a missing skill**. No new artifact warranted; flagged for the retro's
clean-execution score only.

---

## See also

- `.claude/PRPs/handovers/v1-closeout-bootstrap.md` — the as-designed lane
  bootstrap (what the canonical session set up).
- `.claude/PRPs/plans/v1-closeout.plan.md` — the 8-phase close-out plan this
  lane executes.
- `.claude/lessons/feedback_daemon_telegram_completion_hook.md` —
  RECONSTRUCTED this session (GAP 2 fix).
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` —
  the checklist this report recommends amending.
- `project_closeout_no_elitedesk_m1_active.md` (PMD) — the no-EliteDesk
  directive for this lane.
- `.claude/rules/pre-phase-harness-audit.md` — the 4-probe wrapper gate.
