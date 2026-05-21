# v1-dq-schema-r1 planning brief — DQ schema-v3: composite-id + approved_by

**Written**: 2026-05-21 by advisor session (laptop, canonical worktree `C:/Users/barri/Developer/brehon-fork`, branch `governance-v0` @ `8b1aa3645`).
**Subagent target**: `planning` (Opus 4.7 — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/v1-dq-schema-r1-planning-1` from `governance-v0` committed HEAD. Plan file commits + pushes to `governance-v0` at finalize (no phase branch yet — this brief is the pre-phase planning input per advisor-orchestrator.md §2.1 "Planning briefs → committed on `governance-v0`").
**Authority anchor**: issue #142 on `barrie-cork/lemmy`, plus the user-added comment on 2026-05-21T18:32:13Z proposing the composite-id extension. Both are fully quoted in §0 below and are the canonical inputs for this sub-phase.
**Sub-phase target**: `v1-dq-schema-r1` — DQ schema-v3 evolution. Ships **two** schema additions + migration + rule updates: **(A) composite DQ id** (`<session_id>-<sequence>`) to structurally eliminate id collisions (4 recurrences in corpus), **(B) `approved_by` / `approved_at` fields** for judgment-heavy entries to close the audit gap noted by CodeRabbit CR-1 on PR #141.

---

## 0. Why this sub-phase exists — RCA (read first)

### 0.1 Root cause: global monotonic integer id is a shared-counter race

The current `id` field in `.claude/decision-queue.json` is a globally-incremented integer. Every session that writes a new DQ entry computes `next_id = max(all_ids) + 1` from the file on its checkout. Under concurrent multi-lane writes (advisor session A on governance-v0 + advisor session B on phase-v1-<lane> + Junior workers forking from phase tip), the `next_id` computation is a raceable read-modify-write:

1. Session A reads max_id = N, computes N+1.
2. Session B reads max_id = N (before A commits), computes N+1.
3. Both commit with id = N+1 → collision.

**Evidence (4 confirmed collisions):**

| Incident | Sessions | Resolution |
|---|---|---|
| v1-rls-r1 DQ #50 | two worker tasks | Relocation via `e9fa1e01a` — manual, ~5 min |
| v1-fed-in-a | multiple Junior finalize-merges | In-finalize renumber (`feedback_junior_finalize_merge_race_lossless_reconcile.md`) |
| v1-conformance-audit fix-impl cohort | two `[P]` cohort worker tasks | Daemon finalize-merge renumber to avoid #318 collision |
| PR #141 cleanup (2026-05-21) | conformance-audit session vs fed-in-c session | On-merge renumber (#308→#326, #320→#327) in `ca46cfda1` |

Four collisions in one month of multi-lane operation. The recurrence rate matches `feedback_principles_not_rules.md` threshold for a **structural fix** (not another lesson).

### 0.2 Root cause: `answered_by` records resolver, not approval

Per `.claude/rules/decision-queue.md` §"Attribution integrity", `answered_by` records *who resolved* the DQ. For judgment-heavy entries (DQ #307 — scoped as an advisor self-answer; DQ #311 — planner forward-answer), the value is `"advisor-laptop"` or `"planner"`. This doesn't surface whether a user approved the resolution. CodeRabbit CR-1 on PR #141 flagged this gap: entries affecting ADR scope or visible-to-others decisions carry `answered_by: "advisor"` but lack any `approved_by` chain.

**Why it matters:** `feedback_unattributed_security_artifact_reject.md` + advisor-orchestrator.md §3.2 both say judgment-heavy DQs must have user approval. The approval HAS been happening (via AskUserQuestion relay), but the record of approval lives in conversation context — not in the entry itself. An audit that reads only the JSON cannot verify user approval on judgment-heavy entries.

**Scope of the gap (pre-mitigation):** only DQ entries where `kind == "blocker"` AND `answered_by == "advisor"` (not relayed from user) are in scope for `approved_by` backfill. Entries where `answered_by == "user"` already record user approval implicitly. Entries where `answered_by == "impl-self-resolved" | "bm-self-resolved"` are not judgment-heavy (they are self-resolved mechanical findings). Estimated candidates in live file: ~8-12 entries. Migration script identifies them.

### 0.3 Why both changes land in the same sub-phase

Issue #142 (including the user comment 2026-05-21T18:32:13Z) explicitly bundles both:

> "Bundle composite-id design with the `approved_by` / `approved_at` work; both target a single `decision-queue.md` v3 schema spec + one migration commit + one rule update."

The changes share: one schema-version bump (`schema_version: 2` → `3`), one migration script, one rule update to `decision-queue.md`, and updates to every Junior-subagent contract that reads or writes DQ entries. Co-locating them avoids shipping two partial schema bumps that each require a second migration pass.

---

## 0.1 Pre-resolved facts (BINDING — do NOT re-derive or file blockers)

### PRECON-1 — Composite id format: `<session_id>-<sequence>`

**Decision (user comment 2026-05-21, issue #142):** use `<session_id>-<sequence>` where `session_id` is the CC session UUID from `.claude/agent-activity.json` (already tracked) and `sequence` is per-session monotonic. Alternative `<short-sha>-<sequence>` is noted as a tradeoff but the user's comment positions UUID-based as the primary proposal.

**Binding consequences for the plan:**
- New entries write `id: "<session_id>-<seq>"` as the canonical reference. The old integer `id` field is kept as `id_v1: <int>` on historical entries (additive, not a rename — existing scripts parsing `id` as integer keep working on pre-v3 entries).
- **Backward-compat rule for the plan:** pre-v3 entries MUST retain their original integer `id` field. Schema v3 adds `id_v1: <int>` as an alias on migrated entries, and new entries use `id: "<uuid>-<n>"` only. Cross-references in rules/lessons that say "DQ #N" refer to old integer ids for historical entries; new entries are cited as "DQ `<uuid>-<n>`" (or short-form `<4-char-prefix>-<n>`).
- `next_id` computation is abolished. Each session generates its own UUID namespace. No coordination step needed.
- Migration: the script adds `id_v1: e["id"]` to every existing entry, then sets `schema_version: 3`. The integer `id` field is kept as-is (not removed, not renamed) — consumers that read `id` and cast to int keep working on pre-v3 entries; they will fail gracefully on v3 string ids (a runtime error surfaces the migration need, vs silent corruption).

### PRECON-2 — `approved_by` + `approved_at` fields: judgment-heavy scope

**Decision (issue #142 body):** add `approved_by: <user-id> | null` and `approved_at: <ISO8601> | null` (and optionally `approval_note`) to entries flagged judgment-heavy. Alternative `linked_user_dq_id` cross-reference is a tradeoff but PRECON-1 makes the composite id shape cleaner for that field too — the plan names it `linked_dq_ref: <session_id>-<seq> | null` for new entries.

**Binding consequences for the plan:**
- "Judgment-heavy" operational definition for the plan: `kind == "blocker"` AND `answered_by == "advisor"` (advisor self-resolved without explicit user relay). These are the entries the audit gap applies to. Entries already having `answered_by: "user"` are NOT judgment-heavy by this definition.
- Backfill only where evidence exists in git commit history: entries whose resolution commit message matches `^(chore|docs)\((advisor|decision-queue)\)` and the conversation context contains an `AskUserQuestion` exchange. Where no evidence exists, `approved_by: null` stays.
- `approved_by` is populated by the advisor session AFTER a user-gate relay (not by Junior subagents). Hard refusal: no Junior session writes `approved_by`.

### PRECON-3 — Archive files included in migration

**Decision (implicit from issue #142 §"Why deferred" point 1):** "decision-queue.md v2 is forward-only" means historical entries in archive files are NOT rewritten. However, the migration script adds `id_v1` to the **live file** entries only (not archives) since that is the active working set. Archive files keep `schema_version: 1` or `2` as written; the live file becomes `schema_version: 3` after migration.

### PRECON-4 — Rule and subagent doc updates are in-scope

Updates to `.claude/rules/decision-queue.md`, `.claude/agents/impl-task.md`, `.claude/agents/bm-task.md`, `.claude/agents/planning.md`, `.claude/agents/ci-watcher.md` (adding v3 id-format write rules + `approved_by` hard-refusal) are **in-scope** for this sub-phase. These are `.claude/` meta-files, so they ship via PR flow only if a code diff is also present; otherwise they commit direct on governance-v0 per `phase-branch.md` "Direct on governance-v0" policy.

### PRECON-5 — Tooling updates: resolve-dq-canonical.sh + next_id references

`scripts/brehon/resolve-dq-canonical.sh` and every rule/lesson that documents "compute `next_id` via `max(all_ids) + 1`" must be updated to reflect v3 composite id. The `next_id` function is replaced by a per-session `seq` counter initialised at 0 at session start.

---

## 1. Dispatch line (for the Junior planning task — copy verbatim)

```
[role:planning] v1-dq-schema-r1-planning-1 — see .claude/PRPs/briefs/v1-dq-schema-r1-planning-1.md
```

---

## 2. Scope for the planning Junior

The Junior planning task (Opus 4.7) authors `.claude/PRPs/plans/v1-dq-schema-r1.plan.md` following the template at `.claude/PRPs/templates/plan.template.md`.

### 2.1 Deliverable set

The plan covers exactly these deliverables (nothing more, nothing less):

**Track A — Schema spec + migration (`.claude/` meta, ships direct on governance-v0):**
1. `scripts/brehon/dq-schema-v3-migrate.sh` — idempotent migration script. Adds `id_v1` to all live entries, bumps `schema_version` to 3. Dry-run via `--dry-run` flag. Supports `--live-file-only` (skip archives). Outputs a diff summary of changed entries.
2. `scripts/brehon/dq-v3-new-entry.sh` — helper that generates a new v3 entry id (`<session_id>-<seq>`) using the CC session UUID from `.claude/agent-activity.json` (or a fallback UUID if activity file absent/stale). Replaces the `next_id` Python one-liner referenced in `decision-queue.md`.

**Track B — Rule + agent contract updates (`.claude/` meta, ships direct on governance-v0):**
3. `.claude/rules/decision-queue.md` — v3 schema section: composite id format spec, `id_v1` backward-compat rule, `approved_by`/`approved_at` semantics, `linked_dq_ref` shape, hard-refusal additions, `next_id` abolition + `dq-v3-new-entry.sh` recipe. Keep all v2 content intact (forward-only).
4. `.claude/agents/impl-task.md` + `.claude/agents/bm-task.md` + `.claude/agents/planning.md` + `.claude/agents/ci-watcher.md` — add v3 id-format write rule; `approved_by` hard-refusal; updated "Recipe 1" / "Recipe 2" references.

**Track C — `scripts/brehon/resolve-dq-canonical.sh` update:**
5. Update `next_id` logic to v3 (remove max-scan, document the per-session UUID approach). Keep the phase-branch + worker-branch union logic unchanged (still needed for reading, not for id generation).

### 2.2 Explicit NOT building

- The actual composite-id migration of archive files (`.claude/decision-queue-archive-*.json`) — archives keep their original schema per PRECON-3.
- UI or tooling outside the brehon-fork repo.
- Any change to `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`.
- A retroactive backfill of `approved_by` on historical entries where no git evidence of user approval exists.
- The `approval_note` field (optional per issue #142; defer until a concrete need surfaces).
- Any v3 changes to `scripts/brehon/dq-archive.sh` beyond a comment noting schema_version: 3 support (archive policy unchanged).

### 2.3 Plan §13 task structure hint (for the planner)

Suggested task breakdown (planner may refine):

- **Task 0** — pre-flight harness audit (standard; non-`[P]`).
- **Task 1 [P]** — `dq-schema-v3-migrate.sh` + `dq-v3-new-entry.sh` (Track A).
- **Task 2 [P]** — `decision-queue.md` v3 section + 4 agent doc updates (Track B).
- **Task 3** — `resolve-dq-canonical.sh` update (Track C; after Tasks 1+2 committed, to read v3-shaped file correctly).

Tasks 1+2 are `[P]`-able (FILES YAML disjoint: Track A touches only `scripts/brehon/dq-*.sh`; Track B touches only `.claude/rules/decision-queue.md` + `.claude/agents/*.md`). Task 3 depends on both and must be serial.

No `cargo check` / `cargo clippy` / `cargo test` in the DoD — this sub-phase touches zero Rust. DoD is: `bash scripts/brehon/dq-schema-v3-migrate.sh --dry-run` exits 0 on current live file + `bash -n scripts/brehon/resolve-dq-canonical.sh` exits 0 + grep checks on decision-queue.md for `schema_version: 3` section.

---

## 3. Required reading for the planning Junior

- `.claude/rules/decision-queue.md` — current v2 schema (full file; the plan extends it to v3)
- `.claude/rules/advisor-orchestrator.md` — §5.4 DQ triage decision tree (context for `approved_by` semantics)
- `.claude/agents/impl-task.md`, `.claude/agents/bm-task.md`, `.claude/agents/ci-watcher.md` — current DQ write contracts (all four agent files)
- `scripts/brehon/resolve-dq-canonical.sh` — current `next_id` logic (lines to be removed in Track C)
- `.claude/PRPs/templates/plan.template.md` — plan file shape
- `.claude/lessons/feedback_junior_pmd_write_convention.md` — LESSON-trailer discipline
- `.claude/lessons/feedback_parallel_cohort_dispatch.md` — `[P]` cohort rules
- GitHub issue #142 body + user comment (2026-05-21T18:32:13Z) — already reproduced in §0 above; no web fetch needed.

**Canonical schema-first read (per §3.6):** before authoring any new `.claude/rules/` section, read 1-2 existing sections in `decision-queue.md` as the canonical sibling (the "kind: validate-pending" and "ci-watcher mutation pattern" sections are the nearest peers to the v3 composite-id section).

---

## 4. Constraints

- **No content authorship in `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`.** Planner is `.claude/` + `scripts/` meta-only.
- **DQ mid-task push discipline.** Any `kind: "blocker"` raised during planning commits + pushes immediately per `.claude/rules/decision-queue.md` §"Mid-task visibility".
- **LESSON-trailer on every durable finding.** Per `feedback_junior_pmd_write_convention.md` + `.claude/rules/pmd-invariants.md` invariant #4.
- **`[P]` only where FILES YAML disjoint.** Tasks 1+2 must list distinct `creates:` + `modifies:` arrays in the plan §13 YAML block; planner verifies no overlap before marking `[P]`.
- **DoD must be verifiable without cargo.** All §15 DoD commands must be bash/python/grep — no `cargo check`. Plan §15 must name the exact commands; DoD smoke test runs at plan-approval time.
- **Attribution integrity preserved.** The plan must NOT mark migration commits as `answered_by: "advisor"` on pre-existing entries unless a real user gate was recorded. The migration script adds `id_v1` only; it does NOT write `answered_by`.
- **Backward compat invariant.** Any consumer that reads `decision-queue.json` and casts `e["id"]` to int must continue to work on pre-v3 entries after migration. The plan's §12 "NOT building" must enumerate the backward-compat constraint explicitly.

---

## Commit + dispatch plan (advisor side, after brief is authored)

1. `git add .claude/PRPs/briefs/v1-dq-schema-r1-planning-1.md && git commit -m "chore(advisor): author v1-dq-schema-r1 planning brief (issue #142 RCA)" && git push origin governance-v0`
2. Run `/brehon-clarify .claude/PRPs/briefs/v1-dq-schema-r1-planning-1.md` — resolve every clarify-DQ.
3. Surface clarified brief to user for brief-level approval.
4. On approval: dispatch `[role:planning] v1-dq-schema-r1-planning-1 — see .claude/PRPs/briefs/v1-dq-schema-r1-planning-1.md` via `mcp__junior-brehon__create_task`.
