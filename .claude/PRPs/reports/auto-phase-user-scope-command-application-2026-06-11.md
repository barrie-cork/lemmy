# `/auto-phase` User-Scope Command Wiring — Application Report (P50 / Windows)

Plan: `.claude/PRPs/plans/complete-auto-phase-context-management-validation-cleanup.plan.md` (Phase B / B4)
Applied on: **P50** (Windows advisor laptop), host `C:/Users/barri/Developer/brehon-fork`, branch `governance-v0`.
Date: 2026-06-11.

## Why this report exists

The prior validation report (`auto-phase-context-management-validation-2026-06-11.md`)
was executed on a **darwin mirror** (`/Users/barrie/Developer/lemmy`) where the
user-scope command files were absent. It correctly deferred Phases B/D/E/F to
"the Windows advisor laptop, where the command files actually exist." This report
records that deferred application, now completed on P50.

## Command files edited (user-scope — outside the repo, NOT committed)

| File | Existed before? | mtime before | Size before |
|---|---|---|---|
| `~/.claude/commands/auto-phase.md` | yes | 2026-05-22 | 45992 B |
| `~/.claude/commands/compact-phase.md` | yes | 2026-06-04 | 2962 B |

Both files **predated** the entire context-management commit chain (the earliest,
`764dbd1e3 chore(auto-phase): add stage digest ring schema`, landed 2026-06-11).
A grep for every Phase-D marker returned `0` on both files before editing — Phase B
had genuinely never been applied on this host. (The repo-tracked half — schema
template, `refs/auto-phase.md`, `refs/compact-prompt-approach.md` — was already
committed and validated; only the user-scope command bodies remained.)

## Backups

Made before any edit (Phase B1), repo-tracked for audit:

- `.claude/PRPs/reports/user-scope-command-backups/auto-phase.20260611T170529Z.md`
- `.claude/PRPs/reports/user-scope-command-backups/compact-phase.20260611T170529Z.md`

## Sections changed

### `~/.claude/commands/auto-phase.md`

1. **Phase 0.5 Step A — schema-v3 additive backfill** (inserted after the
   `error_class_history` backfill loop, before the `if upgraded:` persist).
   Backfills the schema-v2 ring fields (`stage_digests`, `digest_overflow_path`)
   AND the schema-v3 fields (`spill_dir`, `last_handover_path`, `last_handover_at`)
   with safe defaults derived from `state['phase']`, then bumps `schema_version`
   to `3`. Forward-only, idempotent, single persist. **Deviation from patch note:**
   the patch note only specified the v3 fields; this host's command body lacked the
   v2 ring backfill too (it stopped at `schema_version = 1`), so both v2 and v3
   backfills were added to match `auto-phase-state.template.json`'s
   `_schema_v2_note` + `_schema_v3_note`. Without this the v2 ring fields would be
   absent on any v1→v3 resume and the Phase-1 digest append would `KeyError`.

2. **Phase 1 standing rules** (new `#### Phase 1 standing rules` subsection,
   inserted between the no-op-tick description and the Phase 2 routing table):
   - **Stage-digest ring append** — appends one digest to `stage_digests` after
     each stage transition, trims to `DIGEST_RING_MAX = 12`, spills the oldest to
     `digest_overflow_path` (JSONL). Data-only; MUST NOT leak into Junior briefs.
     **Deviation from patch note:** the patch note said "insert immediately after
     the digest append/write step that was added in Phase 1" — but no such step
     existed in this command body. The append step itself had to be authored (from
     the template's `_stage_digests_note` + `refs/auto-phase.md` invariant #1) as
     the anchor for the handover refresh.
   - **Tool-output spill guard** — >16000-char tool results write full output to
     `spill_dir/<stage>-<tool>-<UTC-iso>.txt`, keep only head+tail+path in context.
     Verbatim from `phase-2-...command-notes.md`.
   - **Auto-handover refresh** — after the digest append, write/overwrite
     `.claude/PRPs/handovers/<phase>-auto-<UTC-date>.md`, set `last_handover_path`
     + `last_handover_at`, commit on `governance-v0`. Verbatim from
     `phase-2-...command-notes.md`.

3. **Phase 0.5 Step E — digest-first resume** (inserted at the top of Step E,
   before the compact-format spec). Builds the resume report primarily from
   `stage_digests[-3:]` + `last_handover_path`; marks the latest
   `next_action_hypothesis` as a HYPOTHESIS to re-verify against live
   TaskList/DQ/PR state. Verbatim from `phase-3-...ledger-plan.md`.

### `~/.claude/commands/compact-phase.md`

4. **Priority 1 — ledger-first active thread** (appended to the priority-1
   "most recent active thread" paragraph). For `/auto-phase` runs (a
   `.claude/auto-state/<phase>.json` ledger exists), cite `stage_digests[-1]` +
   `last_handover_path` verbatim rather than reconstructing from conversation;
   keep next-action a re-verify hypothesis; non-`/auto-phase` sessions unchanged.
   Verbatim from `phase-3-...ledger-plan.md`.

## Validation commands run

### Phase D — command-body grep (all PASS)

| File | Marker | Count |
|---|---|---|
| auto-phase.md | `spill_dir` | 4 |
| auto-phase.md | `last_handover_path` | 4 |
| auto-phase.md | `last_handover_at` | 3 |
| auto-phase.md | `stage_digests[-3:]` | 1 |
| auto-phase.md | `next_action_hypothesis` | 3 |
| auto-phase.md | `schema_version … 3` | 2 |
| auto-phase.md | `DIGEST_RING_MAX` | 2 |
| compact-phase.md | `stage_digests[-1]` | 1 |
| compact-phase.md | `last_handover_path` | 1 |
| compact-phase.md | `re-verify` | 2 |
| compact-phase.md | `next_action_hypothesis` | 1 |

### Embedded-Python dry-run (PASS)

The schema-v3 backfill block + digest-ring block were extracted and executed
against a synthetic v1 state: v1→v3 upgrade adds all 5 fields and is idempotent
on re-run; the ring trims 15 appends to 12 with 3 spilled to overflow in correct
oldest-first order.

### Phase E — runtime simulation (PASS, E.1–E.6)

Ran a disposable phase `validation-auto-phase-context-20260611` end-to-end under
gitignored `.claude/auto-state/`:

- E.1 init v1 state (`schema_version = 1`).
- E.2 Step-A backfill → `schema_version = 3`, 5 fields added, `upgraded = True`.
- E.3 14 stage transitions → ring holds 12, 2 spilled to `<phase>.digests.jsonl`,
  ring tail = `stage-11/12/13`.
- E.4 20000-char tool result → spilled to `<phase>.spill/…txt` (20000 B on disk),
  context reduced to 220 chars (1.1% of original).
- E.5 handover refresh → `<phase>-auto-<date>.md` written, ledger pointers set.
- E.6 resume report built from `stage_digests[-3:]` + `last_handover_path`;
  `next_action_hypothesis` surfaced as re-verify-only.
- gitignore confirmed: `.json`, `.digests.jsonl`, `.spill/` all ignored under
  `.claude/auto-state/`.
- All disposable artifacts removed; working tree clean.

## Deviations summary

1. Added the schema-**v2** ring backfill alongside v3 (patch note specified v3
   only) — required because this host's command body stopped at `schema_version = 1`.
2. Authored the Phase-1 **digest-append step** itself (patch note assumed it
   already existed) — sourced verbatim-equivalent from the repo template +
   `refs/auto-phase.md` invariant #1.

Both deviations are additive, match the committed repo contract, and were
necessary for the patch-note blocks to be coherent on this host.

## Phase F — real dogfood: DEFERRED

A real `/auto-phase` run was not exercised (no live sub-phase is in flight — per
MEMORY.md, m2-late-2 has no plan authored yet). Static + simulation validation
covers the wiring; dogfood will occur naturally on the next real sub-phase. What
remains unverified until then: the digest-append firing inside a real stage
transition, and `/compact` priority-1 reading `stage_digests[-1]` from a live
ledger during an actual compaction.

## Note for the user — PMD topology

While checking PMD freshness, `.mcp.json` still points `project-memory` at
`http://localhost:11435/mcp`. The user noted homeserver is now the PMD server.
The localhost daemon currently answers and holds current data (815 memories,
all 212 lessons synced, the two pi-harness/context lessons indexed as 933/934),
while `homeserver:11435` was not reachable from P50 during this session. The
`.mcp.json` URL was **not** changed (cross-session harness-config change, out of
scope for this `/auto-phase` task) — flagged for the user to decide.
