# Brief: normalise lesson-frontmatter drift + harden the guard

**Status:** SCOPED, not yet dispatched. Authored 2026-06-07 during the MiniMax-AB
close-out session (the trigger incident: two new lessons authored with nested
`metadata.type` were silently skipped by `sync-lessons-to-pmd.sh`).
**Class:** advisor-side harness/tooling task (edits `.claude/lessons/**`, `.claude/hooks/**`,
`scripts/sync-lessons-to-pmd.sh`). NOT a Rust impl-task, NOT a Junior `[role:*]` dispatch —
this is meta-work on `governance-v0`, the kind BM/impl never touch. Run it as an advisor-side
`Agent`-tool dispatch OR a foreground session, on `governance-v0` directly (no phase branch,
no PR — per `phase-branch.md` "Direct on governance-v0" for `.claude/` meta-work).
**Canonical-shape reference:** `.claude/PRPs/briefs/sweep-2026-04-30-c2-harness-cr.md` (harness
sweep) + `.claude/PRPs/briefs/m2-late-1-impl-1.md` (§-section ordering).

---

## §0 The finding (why this exists — read before scoping judgment)

It is NOT two competing conventions. Top-level `type:` is ALREADY the de-facto standard:
- `.claude/lessons/`: **190 top-level `type:`** / 12 nested `metadata.type:` / 6 broken (no parseable type).
- System-1 `~/.claude/projects/.../memory/`: **226 top-level** / 68 nested (OUT OF SCOPE — see §2).

The `sync-lessons-to-pmd.sh` parser (`parse_lesson()`, ~line 159) is a flat `k.partition(':')`
reader — it reads top-level `type:` only and CANNOT see nested YAML. A nested `metadata.type:`
→ empty `type` field → row silently skipped (`imported: 0`, no per-file error). The
`lesson-pmd-sync.sh` PostToolUse hook (awk-based, `in_fm && /^type:/`) ALSO reads top-level
only — so a nested-form lesson reaches PMD via NEITHER path until hand-fixed.

A guard already exists — `.claude/hooks/lesson-frontmatter-reminder.sh` — but it's a
**non-blocking heads-up** that only checks for a non-empty `name:`. The 12 nested lessons have
a valid top-level `name:`, so they PASS the reminder while still FAILING the sync's `type:`
read. That's the gap.

The nested form is taught by the **CLAUDE.md memory-write instruction block** (correct for
System-1 auto-memory; wrong when copied to a `.claude/lessons/` file). Authors who follow that
block for a lesson file produce the drift. The trigger incident: this session authored
`feedback_cheap_model_arm_drops_adr_constraints.md` + `reference_minimax_prompting_best_practices.md`
with nested `metadata.type` — both skipped by the sync, salvaged only because the hook keys on
`file_path` and an explicit re-sync after a manual fix. Already corrected (commit `fc1be386e`);
they are NOT in the 12 below.

## §1 Scope (do EXACTLY this; nothing wider)

Three deliverables, in order:

1. **Fix the 12 nested-`metadata.type` lessons** → rewrite each frontmatter to top-level
   `name:` / `description:` / `type:` (matching the 190-file majority). Preserve the existing
   `name`/`description`/`type` VALUES verbatim — only flatten the structure. All 12 have
   `type=feedback`.
2. **Fix the 6 broken lessons** (no parseable type / missing frontmatter open) → add a correct
   top-level frontmatter block (`name` + `description` + `type: feedback`). Derive `name` +
   `description` from the file's existing H1/first paragraph; do NOT invent content.
3. **Harden `lesson-frontmatter-reminder.sh`** → additionally flag a lesson whose frontmatter
   has `metadata:` with a nested `type:` but NO top-level `type:` (the exact class that slips
   through today). Keep it WARN-not-FAIL (matching its current non-blocking contract) but make
   the WARN name the nested-form problem specifically and cite the top-level fix.
4. **(belt-and-braces, optional but recommended)** Make `sync-lessons-to-pmd.sh` `parse_lesson()`
   tolerate BOTH shapes: if top-level `type:` is absent, fall back to a nested `metadata.type:`
   read. This makes future drift non-fatal to PMD recall while the files get normalised. Do NOT
   remove the top-level read; ADD the fallback.

## §2 Boundaries (HARD — do NOT do these)

- **Do NOT touch System-1 `~/.claude/projects/.../memory/*.md`** (the 68 nested there). That's
  the user's auto-memory, written by a different harness mechanism; migrating it risks fighting
  the harness's own writer. Out of scope by user decision 2026-06-07.
- **Do NOT change the CLAUDE.md memory-write instruction block** — it's correct for System-1.
  (If you want to prevent the copy-to-lesson confusion, the §1.3 reminder-hook WARN is the
  right place, NOT CLAUDE.md.)
- **Do NOT alter any lesson's `name`/`description`/`type` VALUES** — structure-only flatten for
  the 12; content-derive only for the 6.
- **Do NOT touch the 190 already-correct files.**
- **Do NOT open a PR.** `.claude/` meta-work commits direct to `governance-v0` (`phase-branch.md`).

## §3 Required reading (read FIRST)

- `.claude/lessons/feedback_lesson_frontmatter_top_level_type.md` — wait, this is a System-1
  memory; the in-repo equivalent context is THIS brief §0 + the trigger commit `fc1be386e`.
- `scripts/sync-lessons-to-pmd.sh` lines ~156-184 (`parse_lesson()` — the flat parser to make
  tolerant in §1.4).
- `.claude/hooks/lesson-pmd-sync.sh` lines ~59-115 (the awk frontmatter parser — confirm it
  reads top-level only; it does; document whether §1.4's fallback should mirror here too).
- `.claude/hooks/lesson-frontmatter-reminder.sh` (the guard to harden in §1.3 — note its
  existing non-blocking contract + the "8 stranded lessons 2026-05-29" incident it cites).
- `.claude/lessons/feedback_clippy_test_style.md` — a known-GOOD top-level-frontmatter lesson;
  use its `head -6` as the verbatim target shape.
- `.claude/rules/pmd-invariants.md` #4 (LESSON-trailer discipline) + `pmd-search-strategy.md`
  (the sync is the canonical import path).

## §4 The exact file lists (verbatim — do not re-derive, these were enumerated 2026-06-07)

### 12 nested-`metadata.type` → flatten to top-level (all `type=feedback`):
```
feedback_bm_merge_daemon_local_ref_staleness.md
feedback_bootstrap_handover_verified_at.md
feedback_daemon_reset_hard_blocked_use_update_ref.md
feedback_e2e_nextest_filter_groups.md
feedback_envvarguard_audit_window.md
feedback_governance_not_found_opaque_error.md
feedback_lemmy_federation_domain_collision_one_host.md
feedback_nextest_positional_filter_over_dash_e_on_windows.md
feedback_outcome_not_cause_check_retro_bypass.md
feedback_rolling_cumulative_trial_counter.md
feedback_validate_pending_laptop_write_then_stop.md
feedback_workflow_fanout_grep_verify_before_act.md
```

### 6 broken (no parseable type / frontmatter) → add top-level frontmatter:
```
feedback_cherry_pick_onto_restructured_file_reinjects_content.md
feedback_daemon_missing_local_branch_new_lane_cut.md
feedback_daemon_telegram_completion_hook.md
feedback_merge_forward_clippy_debt_from_trunk.md
feedback_merge_forward_e2e_conflict_default_to_governance.md
feedback_mode_b_trunk_phase_sync.md
```
NOTE: several of these are cited BY NAME in always-load rules (e.g.
`feedback_daemon_telegram_completion_hook.md` is referenced in `advisor-orchestrator.md` §1 +
MEMORY.md). Fixing their frontmatter does NOT change their filename or path, so citations stay
valid — but VERIFY each filename is unchanged post-edit.

## §5 Constraints (enforce)

- **Re-verify the count is still 12 + 6 at start** — `awk`-survey both classes before editing
  (this brief's enumeration is dated 2026-06-07; the corpus may have grown). If the count
  drifted, reconcile against §4 and note any new files.
- **Atomic, file-scoped commits** — the canonical checkout is shared with live sessions
  (`multi-lane-worktree.md` hard refusal #6/#7): `git status` first; stage ONLY your files;
  `git commit -F`; `git show --stat HEAD` isolation-verify; one chain. Suggested commits:
  one `fix(lessons): flatten 12 nested-metadata frontmatter to top-level type`, one
  `fix(lessons): add frontmatter to 6 stranded lessons`, one
  `feat(hooks): reminder + sync tolerate/flag nested metadata.type`.
- **Verify the fix end-to-end** — after editing, run `bash scripts/sync-lessons-to-pmd.sh`
  (or `--dry-run`) and confirm the previously-skipped 18 now import (or are already present),
  and `errors:` drops to ~0. Then `memory_search` for one of the 6 previously-broken lessons to
  confirm it's now recallable. (Per `pattern_test_against_reality_not_syntax` — don't trust the
  edit, prove the sync picks it up.)
- **LESSON trailer / retro** — end with a `LESSON:` commit trailer or a session note: the
  durable learning is "one frontmatter convention (top-level type) + a guard that catches the
  nested copy-from-CLAUDE.md class." Do NOT re-author the System-1 memory already written
  (`feedback_lesson_frontmatter_top_level_type.md` in the user's memory dir).
- **Do NOT run cargo** — zero Rust in this task.

## §6 Definition of done

- [ ] 12 nested lessons flattened to top-level `type:` (values preserved).
- [ ] 6 broken lessons have valid top-level frontmatter (filenames unchanged).
- [ ] `lesson-frontmatter-reminder.sh` WARNs on a nested-`metadata.type`-without-top-level-`type`
      lesson, citing the top-level fix.
- [ ] (if §1.4 taken) `sync-lessons-to-pmd.sh` falls back to nested `metadata.type` when
      top-level absent; top-level read unchanged.
- [ ] `bash scripts/sync-lessons-to-pmd.sh` shows the 18 now imported/present, `errors:` ≈ 0.
- [ ] `memory_search` recalls at least one previously-stranded lesson.
- [ ] Commits are file-scoped + isolation-verified; pushed to `governance-v0`; no PR.
