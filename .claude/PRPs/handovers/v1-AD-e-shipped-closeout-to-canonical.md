# v1-AD-e — shipped; close-out commands for the canonical brehon-fork checkout

**Status:** v1-AD-e SHIPPED 2026-05-17. PR #133 merged into `governance-v0` (`486a24c68`), phase branch deleted, ADR-015 fully satisfied, full e2e 94/0/5, `/brehon-verify` 3/3 ✓, retro signed off (gate 6), 3 lessons promoted.

**Why this doc exists:** the remaining 3 close-out items (PMD lesson sync, DQ archive, #292 daemon-fix ticket) require the canonical `C:/Users/barri/Developer/brehon-fork` checkout — they cannot run from the `brehon-fork-ad-e` lane worktree (no `.project-memory/memory.db`; `dq-archive.sh` relative-path assumptions; multi-lane DQ-divergence rule forbids DQ writes from the lane worktree, which is being torn down). The substantive close-out (merge, retro sign-off, lessons promoted+pushed, L14 re-apply) is **already done + pushed**. These three are idempotent housekeeping.

User decisions (2026-05-17): user runs all 3 from homeserver/canonical; #292 ticket goes **Both** (GitHub issue + homeserver troubleshooting-doc pointer).

---

## Already done (this session, all on `governance-v0`)

| Deliverable | Commit |
|---|---|
| PR #133 merged → governance-v0 | `486a24c68` |
| L14 runlog re-apply (BM #301 false-failed at daemon finalize; merge succeeded server-side) | `16e8b2c04` |
| v1-AD-e retro (gate 6 signed off) | `228a881f9` |
| 3 lesson files promoted | `6e6d75b8c` |
| DQ #247 logged (bm-merge 2.5 user-override audit) | `c9da8f79c` (was on phase branch; now on trunk via the #133 merge) |

Phase branch `phase-v1-AD-e` deleted from origin (L16 verified). User gates 1–6 all cleared.

---

## Step 0 — sync the canonical checkout

```bash
cd C:/Users/barri/Developer/brehon-fork
git status --short
```

**Known caveat:** this checkout was on a divergent **detached HEAD `e706cdefe`** during the v1-AD-e session (could not `git pull --ff-only` governance-v0). If `git status` shows detached HEAD or divergence:

```bash
git fetch origin governance-v0
git checkout governance-v0
git pull --ff-only origin governance-v0
```

If `pull --ff-only` **fails (diverged)**: run `git log --oneline -5 governance-v0` + `git log --oneline -5 origin/governance-v0` and investigate before any `reset --hard` (per `.claude/rules/no-destructive-defaults.md` — the local commits may be uncommitted canonical-session work). Target after sync: `governance-v0` at `6e6d75b8c` **or later** (must contain the 3 lessons + the retro).

---

## Step 1 — PMD lesson sync (makes the 3 new lessons searchable)

```bash
cd C:/Users/barri/Developer/brehon-fork
bash scripts/sync-lessons-to-pmd.sh
OLLAMA_URL=http://homeserver:11434 PROJECT_MEMORY_DB=.project-memory/memory.db PROJECT_ROOT=C:/Users/barri/Developer/brehon-fork node C:/Users/barri/Developer/MCPs/project-memory-mcp/dist/scripts/backfill.js --verbose
```

The 3 lesson files (already committed on governance-v0):
- `.claude/lessons/feedback_junior_292_stale_base_recover_recipe.md`
- `.claude/lessons/feedback_impl_task_enumerated_transform_all_or_blocker.md`
- `.claude/lessons/feedback_windows_backslash_path_dq_via_write_fragment.md`

`sync-lessons-to-pmd.sh` adds rows without vectors; the backfill embeds them (idempotent on `(memory_id, model)`). Per `pmd-search-strategy.md`.

---

## Step 2 — DQ archive (228 entries / 614 KB — over the 100-entry / 200-KB threshold; the v1-AD-e retro is the archive trigger per `decision-queue.md`)

**Dry-run FIRST** (cutoff 224 = archive everything before the v1-AD-e era; ids 225–247 stay live as the last-completed phase):

```bash
cd C:/Users/barri/Developer/brehon-fork
bash C:/Users/barri/Developer/homeserver/scripts/dq-archive.sh --dry-run --cutoff-id 224
```

**Before the real run, eyeball the dry-run output and CONFIRM these cited entries are NOT in the archived set** (all are id ≥ 225 so they should be safe — verify):

- **#229** — Shape-G suspended until 2026-06-01 (cited in MEMORY.md + advisor-orchestrator)
- **#247** — bm-merge 2.5 user-override (cited in the v1-AD-e retro)
- **#237 / #238** — v1-AD-e scope/engine decisions (cited in the plan)

(All ≥ 225, above the 224 cutoff — safe. The check is belt-and-braces per `decision-queue.md` "Entries cited by name … stay live".)

If the dry-run archives only `id ≤ 224` and leaves 225–247 live, run for real:

```bash
bash C:/Users/barri/Developer/homeserver/scripts/dq-archive.sh --cutoff-id 224
```

It auto-commits both files as `chore(decision-queue): archive entries up to #224 for v1-AD-e retro`. Then:

```bash
git push origin governance-v0
```

---

## Step 3 — #292 daemon-fix ticket (Both: GitHub issue + troubleshooting-doc pointer)

### 3a — GitHub issue on the fork

```bash
gh issue create --repo barrie-cork/lemmy --title "daemon: Junior workers fork from stale local phase ref (#292 stale-base-self-merge — 5x on v1-AD-e)" --body "$(cat <<'EOF'
## Summary

On a multi-lane phase, Junior workers dispatched on `phase-v1-<lane>` fork the worktree from the **daemon's stale local `phase-v1-<lane>` ref** (which lags `origin`), NOT from `origin/phase-v1-<lane>`. The worker then self-merges the prior abandoned worker's branch and produces a tree whose diff vs the live phase tip is a **mass deletion of every advisor file landed after the stale merge-base**.

**`base_branch=phase-v1-<lane>` on `mcp__junior-brehon__create_task` does NOT prevent it** — the daemon forks from its *local* ref, which the arg does not refresh.

## Evidence

5 consecutive occurrences on v1-AD-e (2026-05-17), effectively every worker:

| Junior | Verb | Deletions if merged |
|---|---|---|
| #292 | bm-pr | -1910 |
| #295 | bm-poll-cr | -766 |
| #296 | bm-triage | -985 |
| #297 | fix-impl-1 | -1496 (+ exited without push) |
| #299 | fix-impl-2 | -2596 |
| #301 | bm-merge | stale worktree (gh pr merge succeeded server-side anyway; daemon finalize false-failed) |

Each required a manual cherry-pick rescue (cherry-pick ONLY the clean commit onto the live phase tip; verify zero advisor-file deletions; reconstruct any validate-pending-laptop DQ pointing at the new on-branch SHA).

## Likely root cause

The `RECOVERY-PASTE-elitedesk.md` `reset --hard` TOCTOU corrupting daemon-local refs, compounding with the daemon-local-vs-origin divergence (`feedback_daemon_local_trunk_stale_multi_lane`).

## Proposed fix

Before each `git worktree add` for a task, the daemon MUST do one of:
- (a) `git fetch origin <phase>:<phase>` — fast-forward the daemon-local phase ref to origin before forking the worktree, OR
- (b) fork the worktree directly from `origin/<phase>` instead of the local ref.

Until this lands, the advisor must budget the recover-cherry-pick as the EXPECTED path for every Junior worker on a multi-lane phase. Recover recipe + detection: `.claude/lessons/feedback_junior_292_stale_base_recover_recipe.md` (governance-v0).

## Distinct variant also seen (#297)

Worker did + committed all work but **exited without `git push`** (its final reasoning got distracted by the mandatory Stop-hook retro requirement; it wrongly assumed Junior finalize would push — finalize skips on no-prepush). Daemon reported `succeeded`. Separate fix: dispatch briefs / impl-task contract must hard-order `git push origin HEAD` BEFORE the Stop-hook retro and forbid exiting between them.

Refs: v1-AD-e retro `.claude/PRPs/reports/v1-AD-e-retro.md`, retros PMD #360/#361/#362/#363/#364.
EOF
)"
```

### 3b — pointer in the homeserver troubleshooting doc

After the issue is created, append a ONE-LINE pointer (referencing the issue #, so the doc stays a pointer not a duplicate) to:

`C:/Users/barri/Developer/homeserver/docs/troubleshooting-laptop-elitedesk.md`

Suggested line (substitute `<ISSUE#>`):

```markdown
- **Junior #292 stale-base-self-merge (multi-lane phases):** workers fork from a stale daemon-local phase ref → mass-delete advisor files. 5x on v1-AD-e. Detection + recover recipe: `brehon-fork:.claude/lessons/feedback_junior_292_stale_base_recover_recipe.md`. Upstream fix tracked: barrie-cork/lemmy#<ISSUE#>.
```

---

## Step 4 — MEMORY.md index entries for the 3 new lessons

Add to `C:/Users/barri/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/MEMORY.md` under the appropriate sections (one line each, ≤150 chars):

Under **## Junior daemon + workers**:
```markdown
- [#292 stale-base recover recipe](feedback_junior_292_stale_base_recover_recipe.md) — workers fork stale daemon-local ref; mass-delete advisor files; 5x v1-AD-e; +#297 no-push variant; recover recipe
```

Under **## Agent / review workflow** (or **## Commit / planning discipline**):
```markdown
- [Enumerated-transform all-or-blocker](feedback_impl_task_enumerated_transform_all_or_blocker.md) — impl-task scrubs obvious subset + reports done; advisor diff-vs-enumeration spot-check is the catch
```

Under **## Windows / cargo wrapper traps**:
```markdown
- [Backslash-path DQ via Write-fragment](feedback_windows_backslash_path_dq_via_write_fragment.md) — \b→backspace mangles scripts\brehon in python -c; author JSON via Write-tool fragment + read-merge
```

(These are PMD-memory-dir edits, not repo edits — they live in the user-memory dir, separate from the lesson files which are repo-tracked on governance-v0.)

---

## Done-when

- [ ] Step 1: `sync-lessons-to-pmd.sh` + backfill run; the 3 lessons searchable via `memory_search_hybrid`
- [ ] Step 2: DQ archived (cutoff 224, dry-run reviewed first), both files committed + pushed
- [ ] Step 3a: GitHub issue created on barrie-cork/lemmy
- [ ] Step 3b: one-line pointer (with issue #) appended to homeserver troubleshooting doc
- [ ] Step 4: 3 MEMORY.md index lines added

After done: v1-AD-e is 100% closed. Remove the lane worktree per `.claude/rules/multi-lane-worktree.md` §3:
```bash
cd C:/Users/barri/Developer/brehon-fork
git worktree remove ../brehon-fork-ad-e
git branch -d phase-v1-AD-e   # local tracking (origin already deleted)
```

The next lane (federation-inbound-a / ship-1) has its own worktree + advisor session — NOT this one's continuation.
