# Decision-queue mechanics

Externalised from `.claude/rules/decision-queue.md` on 2026-05-22
(rule-trim pass). The rule file kept all schema definitions, the kind
enum, hard refusals, attribution integrity, ci-watcher mutation, two-
phase validation under Shape G, and polling-loop routing. This file
holds the operational procedures that fire infrequently (sub-phase
boundaries, schema migrations).

Cite this file as `dq-mechanics.md §"Archive policy"` /
`dq-mechanics.md §"Schema-v3 migration"` etc.

> **Loading note:** this file is NOT auto-loaded at session start.
> Read on-demand when (a) the live DQ approaches the archive trigger,
> (b) authoring a script that touches DQ schema, or (c) handling a
> mixed-vintage archive read.

## Archive policy

The live `decision-queue.json` should hold only entries from **the
current sub-phase + the last-completed sub-phase**. Older entries
move to a sibling archive file. This keeps task-0 reads cheap (the
file is loaded by every Junior subagent at task start) and the
mental model clean ("these are the *current* decisions").

### When to archive

Archive at one of three triggers, whichever fires first:

1. The live file passes **100 entries** or **200 KB**, OR
2. A Junior subagent's task-0 read is observed dropping content due
   to context pressure, OR
3. **At sub-phase retro** — the natural close-out moment. Archive
   everything that doesn't reference the new sub-phase. This is the
   primary trigger; the others are safety nets.

As of 2026-04-27 (post-v1-JM-c ship): live file holds 51 entries,
~128 KB. Comfortable. First archive landed at pre-v1-AD-e (see
`decision-queue-archive-pre-v1-AD-e.json`).

### How to archive

Use `homeserver/scripts/dq-archive.sh` (idempotent, supports
`--dry-run`). It moves resolved entries with `id <= --cutoff-id` from
the live file to a dated archive file at
`.claude/decision-queue-archive-<sub-phase-slug>.json` (e.g.
`decision-queue-archive-pre-v1-AD-e.json`). The script:

- Reads both `pending` and `resolved`; only resolved entries with
  `id <= cutoff` are eligible (pending entries are never archived).
- Preserves entry shape exactly — does NOT rewrite historical
  idiosyncrasies (per the v2 forward-only rule in decision-queue.md).
- Sets `schema_version` on the archive file to **the value the
  archived entries used**, not the live file's version. If the
  archive contains v1 entries (no `kind`, drift in resolved-timestamp
  keys), the archive file is `schema_version: 1`. Mixed-vintage
  archives use `schema_version: 1` (the lower of any contained version)
  so consumers know to handle drift.
- Commits both files in one commit with subject
  `chore(decision-queue): archive entries up to #<cutoff-id> for <sub-phase> retro`.

### Reading across live + archive

When a subagent needs an entry by id and it's not in the live file,
check the archive(s):

```bash
ls .claude/decision-queue-archive-*.json
# Read each until the id is found
```

### What does NOT get archived

- **Pending entries** — never. Pending entries are by definition
  blocking current work.
- **Entries cited by name** in any active rule, lesson, brief, or
  template. If `decision-queue.md` references DQ #37 by id (it does),
  that entry stays live as long as the citation does. Move the
  citation to the archive file path before archiving the entry.
- **Entries from the active sub-phase or its predecessor.** These are
  load-bearing for the current advisor session and any subagent that
  may need to revisit recent decisions.

## Schema-v3 migration (historical)

Schema-v3 is an additive, forward-only extension of schema-v2 shipped
under v1-dq-schema-r1 (2026-05-21). The `dq-schema-v3-migrate.sh`
migration adds two fields to every existing entry and bumps
`schema_version` to `3`:

- `id_v1: <int>` — alias for back-compat readers (resolves "DQ #50"
  citations against the legacy integer `id` field).
- `approved_by: null` + `approved_at: null` — advisor-exclusive
  user-gate approval fields (populated only on AskUserQuestion relay).

Archive files (`decision-queue-archive-*.json`) are **NOT migrated** to
v3 — they retain schema_version 1 or 2 per PRECON-3 (forward-only;
archive migration is permanently deferred).

The composite-id mechanism (`<session_id>-<sequence>` per CC session)
replaces the pre-v3 `next_id = max(all_ids) + 1` recipe. Pre-v3 entries
keep their integer ids unchanged; new v3 writes use
`bash scripts/brehon/dq-v3-new-entry.sh` to generate the composite id.
See decision-queue.md §"Schema (v3)" for the live contract.

### Pre-v3 next-id calculation (historical only)

Pre-v3 entries used integer ids computed as
`max(all_ids, default=0) + 1` spanning the live DQ + archive files.
That recipe is **abolished** for new v3 writes (per decision-queue.md
Hard refusal #9) — using it on a mixed int/string id corpus raises
`TypeError` on Python `max()`. Do **not** copy the historical recipe
into a new authoring script. The pre-v3 recipe survives only so the
`dq-schema-v3-migrate.sh` provenance is readable; it is not a recipe
to re-implement.

The DQ #50 collision incident (`e9fa1e01a`) and the recurring
cross-lane race documented in `feedback_cohort_dq_id_collision.md` are
both structurally eliminated by the v3 composite-id mechanism.

## Deprecated kinds (historical-only — do not use for new writes)

`kind: "validate-result"` and `kind: "validate-failed"` are DEPRECATED
2026-04-28 by option 2 (single-entry mutation). No session writes them.
Schema-v2 readers tolerate historical entries (DQ #74 on `governance-v0`
carries `validate-result`; paired with manually-migrated DQ #73 at
`30597b436`). See decision-queue.md Hard refusal #7.

**Note on enum values:** GitHub's workflow `conclusion` API returns
`timed_out` (with underscore). `run_not_found` covers garbage-collected
or wrong-branch runs; ci-watcher's pre-flight check mutates the paired
entry with this result and exits 0.

## Two-phase validation under Shape G (option (b), locked 2026-04-28)

> Shape G is SUSPENDED until 2026-06-01 (minutes exhausted; the
> validate-pending-laptop handler is active in the interim). This section
> documents the locked Shape-G dispatch flow for when it re-enables.

Per `cargo-test-e2e.yml` triggering on push to `phase-v1-*` only (not
`junior/*` worktree branches — option (b) defers e2e to the phase-branch
tip; saves ~80% of e2e runs across a sub-phase):

- **Phase 1 (workspace check on `junior/*`):** impl-task writes a
  `validate-pending` entry referencing the `cargo-validate-workspace.yml`
  run id. Goes to `pending`, `from: "impl"`. Advisor queues a ci-watcher
  to mutate it.
- **Phase 2 (e2e on `phase-v1-*`):** after Junior's daemon finalize-
  merges the impl-task worktree branch into the phase branch, the
  advisor's polling loop detects the new phase-branch tip on next
  `git fetch`. The push to `phase-v1-*` triggers `cargo-test-e2e.yml`.
  The advisor captures the e2e workflow_run_id via `gh run list --repo
  barrie-cork/lemmy --branch phase-v1-<phase> --workflow cargo-test-e2e
  --limit 1 --json databaseId`, and writes a NEW `validate-pending`
  entry, `from: "advisor"` (commit subject `chore(advisor): raise e2e
  validate-pending for phase-v1-<phase> tip <sha>` per
  `^(chore|docs)\((advisor|decision-queue)\)`). Advisor queues a second
  ci-watcher to mutate it.

Both phases use the same single-entry mutation pattern. Cohort
advancement waits on both phases per the cohort dispatch rule in
`.claude/rules/advisor-orchestrator.md`.

## See also

- `.claude/rules/decision-queue.md` — live schema, kind enum,
  hard refusals, attribution, ci-watcher mutation, two-phase
  validation, polling-loop routing.
- `scripts/brehon/dq-schema-v3-migrate.sh` — the migration script.
- `scripts/brehon/dq-v3-new-entry.sh` — composite-id generator.
- `scripts/brehon/dq-v3-append-fragment.sh` — fragment-append helper.
- `feedback_cohort_dq_id_collision.md` — historical bug class
  structurally fixed by v3.
