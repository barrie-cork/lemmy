# `/auto-phase` Context-Management Validation Report — 2026-06-11

Plan: `.claude/PRPs/plans/complete-auto-phase-context-management-validation-cleanup.plan.md`
Executed on: `darwin` host `/Users/barrie/Developer/lemmy` (branch `work/governance-v0`).

> **CLOSED OUT 2026-06-11 on P50 (Windows).** The Phases B/D/E/F deferral below was
> resolved on the Windows advisor laptop, where the user-scope command files exist.
> All five insertion blocks are applied and validated (Phase D grep PASS, Phase E
> runtime sim PASS E.1–E.6). See
> `.claude/PRPs/reports/auto-phase-user-scope-command-application-2026-06-11.md`
> for the application + validation record. Phase F (real dogfood) remains deferred
> to the next live sub-phase.

## Executive summary

The repo-tracked half of the `/auto-phase` context-management chain (schema-v3
fields, gitignore coverage, digest-first compact/resume docs) **validates clean**.
The user-scope half (Phases B/D/E/F — applying and validating the command-file
wiring) is **blocked on this machine**: `~/.claude/commands/auto-phase.md` and
`~/.claude/commands/compact-phase.md` do not exist here (the `~/.claude/commands/`
directory is absent entirely). Phase G/H cleanup is **moot** — every artifact the
plan listed as "modified" or "untracked" is already committed and the working tree
is clean.

## Phase A — Preflight (PASS)

- `git status --short` → empty (clean tree).
- HEAD: `d431c2154 chore(pi-harness): context injection refactor — progressive disclosure by role`
  — a later intentional successor to `5512e50e2 chore(auto-phase): implement compact ledger wiring`.
  The full auto-phase chain is present: `764dbd1e3` (schema) → `529aae6dc`
  (spill+handover) → `328991851` (compact resume) → `5512e50e2` (ledger wiring).
- No `.pi/**` changes in the working tree (clean).

## Phase C — Static validation (PASS)

| Check | Result |
|---|---|
| `auto-phase-state.template.json` parses | ✅ |
| `schema_version == 3` | ✅ |
| Phase 1/2 fields present (`stage_digests`, `digest_overflow_path`, `spill_dir`, `last_handover_path`, `last_handover_at`) | ✅ all 5 |
| `stage_digests` is a list | ✅ |
| `.claude/auto-state/` gitignored (state, `.spill/`, `.digests.jsonl`) | ✅ all three resolve to `.gitignore:76` |
| `stage_digests[-1]` cited in `compact-prompt-approach.md` + `phase-3-...plan.md` | ✅ |
| `stage_digests[-3:]` cited in `refs/auto-phase.md` + `phase-3-...plan.md` | ✅ |
| `last_handover_path` in compact-prompt-approach.md (2×) + refs/auto-phase.md (3×) | ✅ |
| `next_action_hypothesis` in compact-prompt-approach.md (3×) + refs/auto-phase.md (2×) | ✅ |
| `re-verify` in compact-prompt-approach.md (3×) + refs/auto-phase.md (2×) | ✅ |

The repo docs consistently document digest-first compact/resume behavior with
`next_action_hypothesis` marked re-verify-only. `compact-prompt-approach.md`
already carries the 2026-06-11 optimisation-history entry for digest-first
`/compact` wiring.

## Phases B / D / E / F — Command-file wiring (BLOCKED on this host)

The user-scope command files the plan operates on are **absent** on this machine:

```
$ ls ~/.claude/commands/auto-phase.md ~/.claude/commands/compact-phase.md
ls: /Users/barrie/.claude/commands/auto-phase.md: No such file or directory
ls: /Users/barrie/.claude/commands/compact-phase.md: No such file or directory
$ ls ~/.claude/commands/
ls: /Users/barrie/.claude/commands/: No such file or directory
```

### Why this is a host mismatch, not a missing artifact to create here

1. **The command bodies are user-scope and were never tracked in the repo.** The
   original `d078f6a37 feat(advisor): /auto-phase skill` commit added only the
   in-repo rule (`.claude/rules/auto-phase.md`, later relocated to
   `.claude/refs/auto-phase.md`) + the state template + supporting edits. The
   ~250-line skill *body* lives only at `~/.claude/commands/auto-phase.md` on the
   Windows advisor laptop. `git log --all -- '**/commands/auto-phase.md'` returns
   nothing — confirmed never in repo history.
2. **The plan + all refs target `C:/Users/barri/` (Windows).** This checkout is
   `/Users/barrie/` (darwin) — a repo mirror. The surrounding harness
   (`~/.claude/commands/`, the `brehon-fork` canonical checkout, the PMD db) is on
   the Windows host.
3. **The patch-note reports carry only insertion blocks, not full bodies.**
   `phase-2-...command-notes.md` and `phase-3-...ledger-plan.md` contain the 3
   auto-phase insertion blocks + the compact-phase priority-1 + Step-E blocks —
   not the complete skill body. A `~/.claude/commands/auto-phase.md` reconstructed
   from these alone would be a broken, partial skill (insertion blocks with no
   surrounding state machine). Creating it would be actively harmful.

**Decision: Phases B/D/E/F are deferred to the Windows advisor laptop**, where the
command files actually exist. The exact insertion blocks are already captured,
repo-tracked, and ready to apply there:
- `~/.claude/commands/auto-phase.md`: 3 blocks from `phase-2-auto-phase-spill-handover-command-notes.md`
  (Phase 0.5 schema-v3 backfill, Phase 1 spill guard, Phase 1 auto-handover refresh)
  + Phase 0.5 Step E block from `phase-3-auto-phase-compact-ledger-plan.md`.
- `~/.claude/commands/compact-phase.md`: priority-1 active-thread block from
  `phase-3-auto-phase-compact-ledger-plan.md`.

When applied on the Windows host, run the Phase D grep checks there to confirm.

## Phase E / F — Runtime + dogfood validation (DEFERRED)

Cannot run without the command files. Deferred to the Windows host alongside
Phases B/D. The repo-tracked schema is verified parseable and complete (Phase C),
so the runtime simulation's preconditions (schema-v3 fields, gitignored runtime
paths) are confirmed sound.

## Phase G / H — Cleanup (MOOT — already committed)

Every artifact the plan listed as modified/untracked is **tracked and committed**;
the working tree is clean. Confirmed via `git ls-files --error-unmatch`:

- `.claude/PRPs/templates/plan.template.md` — tracked
- `scripts/brehon/pmd-query.sh`, `scripts/brehon/pmd-write.sh` — tracked (committed `82310dcf4`, `81f3d2b60`)
- `.claude/PRPs/plans/brehon_context_management_review.md` — tracked
- `.claude/PRPs/plans/claude_auto_phase_context_patch_plan.md` — tracked
- `.claude/PRPs/plans/pi-harness-context-injection.plan.md` — tracked
- `docs/research/brehon_context_management_review.md` — tracked
- `docs/research/claude_auto_phase_context_patch_plan.md` — tracked
- pi-harness reports + lessons (`feedback_pi_monolithic_context_dump.md`,
  `reference_pi_progressive_disclosure_by_role.md`, the two pi-harness retros) — tracked

The pi-harness artifacts were committed in `d431c2154 chore(pi-harness): context
injection refactor`. No cleanup decisions remain; no `.pi/**` staging risk exists
(tree clean).

## Done-criteria status

| Criterion | Status |
|---|---|
| User-scope `/auto-phase` + `/compact` files contain Phase 2/3 blocks | ⏸ deferred to Windows host (files absent here) |
| Schema parses and remains version 3 | ✅ |
| Runtime state/spill/digest paths gitignored | ✅ |
| Validation report records static checks + simulated/real dogfood | ✅ (this report; dogfood deferred) |
| Final commits exclude unrelated changes and `.pi/**` | ✅ (tree clean; nothing unrelated to stage) |
| Working tree clean or documented deferrals | ✅ (clean; deferrals documented above) |
