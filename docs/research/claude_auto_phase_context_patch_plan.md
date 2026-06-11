# Claude Code `/auto-phase` — Exact Context-Management Patch Plan

**Status:** REVIEW / PLANNING ONLY. No repository files modified, no patches applied, no commits created. This document is the patch plan; the diffs below are proposals to be applied in a later, separate change.

**Scope:** Claude Code harness only (`.claude/**`, plus user-scope command notes for `~/.claude/commands/auto-phase.md` and `~/.claude/commands/compact-phase.md`). `/auto-phase` only. No Pi Coding (`.pi/`, AGENTS.md) recommendations.

**Repo snapshot inspected (exact line contents grounding every diff below):**
- `.claude/PRPs/templates/auto-phase-state.template.json` — 86 lines, schema_version 1.
- `.claude/refs/auto-phase.md` — 685 lines (authoritative in-repo spec; user-scope skill body NOT in snapshot).
- `.claude/refs/compact-prompt-approach.md` — 127 lines (tracked rationale for the user-scope `/compact` prompt).
- `.gitignore` line 76: `.claude/auto-state/` (whole dir ignored — no new ignore entry needed for sibling runtime files).
- `~/.claude/commands/auto-phase.md` and `~/.claude/commands/compact-phase.md` — **CONFIRMED ABSENT** from the repo snapshot (user-scope). Per constraint #7, section 7 provides insertion points + exact text blocks, NOT exact diffs.

---

## 1. Executive summary — smallest safe implementation path

The `/auto-phase` ledger (`.claude/auto-state/<phase>.json`) already survives every session boundary and is the one durable artifact a cold resume reads. Every context-management improvement should ride on that existing, proven substrate rather than introduce a new mechanism. The smallest safe path is three additive phases, each independently shippable and independently revertible:

- **Phase 1 — Stage digest ring (ledger-only).** Add a bounded `stage_digests` array to the state schema. On each stage transition the skill appends a ~5-line digest (stage, ts, one-line outcome, key ids, next-action hypothesis) and trims to the last N (default 12). Overflow spills to a gitignored JSONL sibling. This is pure data-shape + one write step; it changes no routing, no gates, no cadence. It is the foundation the other two phases lean on.

- **Phase 2 — Uniform tool-output spill guard + auto-handover refresh.** Generalise the size mitigations that already exist per-tool (PMD 85K–105K spill footgun, `log_slice` ~100–200-line cap, cargo-output-capture) into one rule: any tool result over a threshold is written to a gitignored spill file under `.claude/auto-state/<phase>.spill/` and only a head+tail+path is kept in context. Pair it with an auto-emitted handover refresh: on each stage transition the skill refreshes `.claude/PRPs/handovers/<phase>-auto-<date>.md` from the latest digest, so the pre-compact handover discipline (already mandated in `advisor-orchestrator.md`) is satisfied automatically instead of by hand.

- **Phase 3 — `/compact` wiring + resume refinement.** Teach the custom `/compact` prompt to cite the latest ledger digest + handover path as priority-1 source (instead of reconstructing the active thread from conversation), and refine Phase 0.5 resume to read `stage_digests` as the self-contained "what was happening" narrative — shrinking the COMPACT resume report's reliance on conversation memory that, by design, does not survive.

**Why this ordering is safe:** Phase 1 is data-only and inert until something reads it; Phases 2 and 3 are the readers. Each phase is gated behind a `schema_version` bump so a resume that hits an old ledger backfills the new fields to empty (the existing Phase 0.5 Step A "schema upgrade" path already does this for `error_class_history`). Nothing changes the six user gates, catch-fire terminality, cohort barriers, DQ attribution, or cadence invariants.

---

## 2. Files to change and why

| File | In repo? | Change class | Why |
|---|---|---|---|
| `.claude/PRPs/templates/auto-phase-state.template.json` | Yes | Additive schema + version bump | Add `stage_digests`, `digest_overflow_path`, `spill_dir`, `last_handover_path`, `last_handover_at`; bump `schema_version` 1→2. The ledger is the only durable resume substrate; all three phases store/read here. |
| `.claude/refs/auto-phase.md` | Yes | Additive doc | Document the digest ring under §"Skill ownership boundaries" WRITES + §"What survives across session boundaries"; document the spill guard + auto-handover as new writes; add a §"Context-management invariants" block so the rules are load-bearing, not just descriptive. This is the authoritative in-repo spec the user-scope skill body cites. |
| `~/.claude/commands/auto-phase.md` | **No (user-scope)** | Insertion points + text blocks (§7) | The behavioral body: the actual digest-write step on each transition, the spill-guard wrapper, the handover-refresh step. Not in snapshot → text blocks, not diffs. |
| `~/.claude/commands/compact-phase.md` | **No (user-scope)** | Insertion points + text blocks (§7) | The custom `/compact` prompt: add "cite latest ledger digest + handover path" as priority-1 source. Not in snapshot → text blocks. |
| `.claude/refs/compact-prompt-approach.md` | Yes | Additive doc (optional, Phase 3) | The tracked rationale for `compact-phase.md`; the doc's own rule ("If you edit the command file, update this note in the same session") means a Phase-3 `compact-phase.md` edit must be recorded here. |
| `.gitignore` | Yes | **No change needed** | Line 76 `.claude/auto-state/` already ignores the whole dir; `<phase>.digests.jsonl` and `<phase>.spill/` are siblings inside it. Confirmed by grep. |

---

## 3. Phase 1 patch plan — stage digest ring only

**Goal:** a bounded, durable, append-on-transition ring of compact stage digests in the ledger, so resume and `/compact` have a self-contained narrative that does not depend on conversation memory.

**Behavior:**
1. On every stage transition (the skill already writes the ledger on each transition), append one digest object to `stage_digests` (shape in §8).
2. After append, if `len(stage_digests) > DIGEST_RING_MAX` (default 12), shift the oldest entries to the overflow JSONL file at `digest_overflow_path` (one JSON object per line, append-only) and keep only the last 12 in the ledger.
3. `digest_overflow_path` defaults to `.claude/auto-state/<phase>.digests.jsonl` (gitignored sibling).
4. Phase 0.5 resume reads `stage_digests` as the "here's what was happening" source for the COMPACT resume report.

**Invariants preserved:** ring is data-only; it does not gate, route, or change cadence. The existing catch-fire `.md` dump remains the only *other* non-JSON runtime write; the digest JSONL is additive and equally gitignored. `schema_version` bumps 1→2 so older ledgers backfill `stage_digests: []` + null path fields on resume (mirrors the existing `error_class_history` backfill in Phase 0.5 Step A).

**Files touched in Phase 1:**
- `.claude/PRPs/templates/auto-phase-state.template.json` (§6 diff A).
- `.claude/refs/auto-phase.md` (§6 diff B — the WRITES list + new invariants block).
- `~/.claude/commands/auto-phase.md` (§7 block 1 — the per-transition digest-write step).

**Why ship this first:** zero behavioral risk (inert until read), and it is the data dependency for Phases 2 and 3.

---

## 4. Phase 2 patch plan — tool-output spill guard + auto-handover refresh

**Goal A — uniform spill guard.** Generalise the per-tool size mitigations already in the codebase (PMD broad-search 85K–105K spill per `pmd-search-strategy.md`; DQ `log_slice` ~100–200-line cap; `cargo-output-capture`) into one rule the skill applies to any large tool result during orchestration.

**Behavior:**
1. Define `SPILL_THRESHOLD_CHARS` (default 16000) and `spill_dir` (default `.claude/auto-state/<phase>.spill/`).
2. When a tool result exceeds the threshold, write the full result to `spill_dir/<stage>-<tool>-<UTC-iso>.txt`, keep only head (first ~40 lines) + tail (last ~40 lines) + the spill path in context, and record a spill record (shape in §8) — either inline in the relevant digest or as a line in a `<phase>.spill.jsonl` index.
3. This is the same discipline `pmd-search-strategy.md` already recommends (run broad semantic queries in a subagent so the dump stays out of main context); the guard makes it uniform and automatic rather than per-tool memory.

**Goal B — auto-handover refresh.** `advisor-orchestrator.md` §1 already *mandates* a self-contained handover at `.claude/PRPs/handovers/<phase>-<scope>-<date>.md` before any `/compact`/session-end boundary. Today that is hand-authored. Automate it:

**Behavior:**
1. On each stage transition, after the digest append (Phase 1), the skill writes/refreshes `.claude/PRPs/handovers/<phase>-auto-<UTC-date>.md` from the latest digest + the durable ledger fields (current sub-phase, stage, last commit on the relevant branch, next concrete action, DQ pending ids, concurrent-activity note).
2. Record `last_handover_path` + `last_handover_at` in the ledger.
3. The handover file is committed on `governance-v0` per the existing pre-compact discipline — **but** since the skill must not author a `.md` outside `.claude/auto-state/` as tracked content lightly, the auto-handover is written under the existing handovers path (already a tracked, advisor-owned location per CLAUDE.md canonical paths `bootstrap`/handovers) and committed with a `chore(advisor):` subject matching the attribution pattern. This stays inside the skill's existing WRITES contract for briefs/runlog/auto-state and the `chore(advisor):` commit-subject rule.

**Invariants preserved:** spill files are gitignored (under `.claude/auto-state/`); the handover path + commit subject already conform to the skill's WRITES + attribution rules. No gate/route/cadence change.

**Files touched in Phase 2:**
- `.claude/PRPs/templates/auto-phase-state.template.json` (`spill_dir`, `last_handover_path`, `last_handover_at` — already added in §6 diff A as one combined schema bump, so Phase 2 needs no further template change if diff A is applied whole; if diff A is split, the spill/handover fields land here).
- `.claude/refs/auto-phase.md` (§6 diff B — the spill guard + auto-handover entries in the WRITES list and invariants block).
- `~/.claude/commands/auto-phase.md` (§7 block 2 — the spill-guard wrapper; §7 block 3 — the handover-refresh step).

---

## 5. Phase 3 patch plan — `/compact` wiring + resume refinement

**Goal:** make the durable ledger digest + handover the priority-1 *source* for both `/compact` summarisation and Phase 0.5 resume, so the summary spends its budget on the irreducible active thread and stops trying to reconstruct it from conversation memory that does not survive.

**Behavior:**
1. **`/compact` wiring** (user-scope `compact-phase.md`, §7 block 4): add to priority 1 ("active thread") an instruction to read `.claude/auto-state/<phase>.json` `stage_digests[-1]` + `last_handover_path` and cite them as the authoritative active-thread state, rather than reconstructing branch/stage/in-flight from conversation. This dovetails with the doc's existing priority-1 wording and the lane-vocabulary fix.
2. **Resume refinement** (`auto-phase.md` Phase 0.5, §7 block 5 + doc in §6 diff B): Phase 0.5 Step E builds the COMPACT resume report (≤15 lines / ≤500 tokens) primarily from `stage_digests` + `last_handover_path`, keeping the lazy-load + subagent-delegation discipline already documented in `.claude/refs/auto-phase.md` §"Resume semantics". The next-action carried from the latest digest is marked a **hypothesis to re-verify** (per `feedback_thin_wakeup_prompts_verify_live_state.md`, already cited in `compact-prompt-approach.md`).
3. **Doc sync:** update `.claude/refs/compact-prompt-approach.md` optimisation history with the Phase-3 change (the doc's own discipline requires this when `compact-phase.md` changes).

**Invariants preserved:** Phase 0.5 stays read-only until user 'continue' (hard invariant B); session_id still rotates (C); the resume report stays COMPACT; lazy-load discipline unchanged. The digest is *additional* signal, not a new gate.

**Files touched in Phase 3:**
- `~/.claude/commands/compact-phase.md` (§7 block 4).
- `~/.claude/commands/auto-phase.md` (§7 block 5 — Phase 0.5 digest-first resume).
- `.claude/refs/compact-prompt-approach.md` (§6 diff C — optimisation-history entry).
- `.claude/refs/auto-phase.md` (§6 diff B already documents the resume read).

---

## 6. Proposed unified diff blocks (repository files only)

> These are **proposals**, grounded in the exact current file contents inspected this session. They are plausible and specific enough to apply later. **Do not apply now.** Line anchors use the current file content; apply with context-matching, not absolute line numbers.

### Diff A — `.claude/PRPs/templates/auto-phase-state.template.json`

Additive fields + `schema_version` bump. Anchors: the `_schema_note`/`schema_version` head (lines 2–3) and the `catch_fire`/`_catch_fire_shape` tail (lines 79–86).

```diff
--- a/.claude/PRPs/templates/auto-phase-state.template.json
+++ b/.claude/PRPs/templates/auto-phase-state.template.json
@@
-  "_schema_note": "Runtime state for /auto-phase skill. File written to .claude/auto-state/<phase>.json (gitignored). Updated every tick. Read at session resume to route to current stage handler.",
-  "schema_version": 1,
+  "_schema_note": "Runtime state for /auto-phase skill. File written to .claude/auto-state/<phase>.json (gitignored). Updated every tick. Read at session resume to route to current stage handler.",
+  "schema_version": 2,
+  "_schema_v2_note": "v2 (additive, forward-only) adds the stage-digest ring (stage_digests + digest_overflow_path), the tool-output spill guard (spill_dir), and the auto-handover pointers (last_handover_path + last_handover_at). Phase 0.5 Step A backfills all v2 fields to their empty defaults ([] / null) on resume against a v1 state file, mirroring the error_class_history backfill. No v1 field semantics change.",
   "phase": "v1-SL-c-2",
@@
   "catch_fire": null,
   "_catch_fire_shape": {
     "stage_when_fired": "impl-cohort-3-validating",
     "reason": "non-allowlist §G4 fail",
     "log_slice_path": ".claude/auto-state/v1-SL-c-2-catchfire-20260507-1830.md",
     "fired_at": "2026-05-07T18:30:00Z"
-  }
+  },
+  "stage_digests": [],
+  "_stage_digests_note": "Bounded ring (last DIGEST_RING_MAX=12 entries) of compact per-transition digests. Appended on every stage transition AFTER the ledger's stage field is updated. Each entry: {stage, entered_at, outcome (one line), key_ids (cohort/junior/dq/pr ids touched this stage), next_action_hypothesis}. When the ring exceeds 12, the oldest entries are appended (one JSON object per line) to digest_overflow_path and dropped from this array. Read by Phase 0.5 Step E to build the COMPACT resume report and by /compact (compact-phase.md priority 1) as the authoritative active-thread source. Data-only: does NOT gate, route, or change cadence.",
+  "digest_overflow_path": ".claude/auto-state/v1-SL-c-2.digests.jsonl",
+  "_digest_overflow_note": "Append-only JSONL overflow for stage_digests beyond the ring cap. Gitignored sibling under .claude/auto-state/ (covered by the existing .gitignore line '.claude/auto-state/'). Path is illustrative; the skill sets it to .claude/auto-state/<phase>.digests.jsonl.",
+  "spill_dir": ".claude/auto-state/v1-SL-c-2.spill/",
+  "_spill_dir_note": "Directory for the uniform tool-output spill guard. Any tool result over SPILL_THRESHOLD_CHARS (default 16000) is written here as <stage>-<tool>-<UTC-iso>.txt; only head+tail+path are kept in context. Generalises the per-tool size mitigations (PMD broad-search spill per pmd-search-strategy.md, DQ log_slice cap, cargo-output-capture). Gitignored sibling under .claude/auto-state/. Path is illustrative; the skill sets it to .claude/auto-state/<phase>.spill/.",
+  "last_handover_path": null,
+  "_last_handover_note": "Path to the most recent auto-emitted handover (refreshed on each stage transition from the latest stage_digests entry + durable ledger fields). Default location .claude/PRPs/handovers/<phase>-auto-<UTC-date>.md (tracked, advisor-owned per CLAUDE.md canonical handovers path; committed with a chore(advisor): subject). Satisfies the advisor-orchestrator.md §1 pre-compact handover discipline automatically. last_handover_at is the ISO8601 of the last refresh.",
+  "last_handover_at": null
 }
```

### Diff B — `.claude/refs/auto-phase.md`

Three additive edits: (B1) extend the WRITES list under §"Skill ownership boundaries"; (B2) add a new §"Context-management invariants" block; (B3) extend §"What survives across session boundaries".

**B1 — WRITES list (anchor: lines 92–110, the `The skill **WRITES**:` block; insert after the `.claude/runlog/bm-runlog.md` bullet, before the "The advisor's own commit subjects" bullet):**

```diff
--- a/.claude/refs/auto-phase.md
+++ b/.claude/refs/auto-phase.md
@@
 - `.claude/runlog/bm-runlog.md` — one-line `## advisor: auto-phase
   …` entries on stage transitions for the durable audit trail (also
   appends `docs(advisor)` blocks for the L14 belt-and-braces fallback
   if a BM Junior skips the runlog commit).
+- `.claude/auto-state/<phase>.digests.jsonl` — append-only overflow
+  for the `stage_digests` ring once it exceeds the cap (schema-v2,
+  gitignored sibling under `.claude/auto-state/`).
+- `.claude/auto-state/<phase>.spill/<stage>-<tool>-<ts>.txt` —
+  uniform tool-output spill guard target (schema-v2, gitignored
+  sibling; head+tail+path kept in context, full output spilled here).
+- `.claude/PRPs/handovers/<phase>-auto-<date>.md` — auto-emitted,
+  refreshed on each stage transition from the latest digest +
+  durable ledger fields; satisfies the `advisor-orchestrator.md` §1
+  pre-compact handover discipline automatically. Tracked,
+  advisor-owned; committed with a `chore(advisor):` subject per the
+  attribution-integrity pattern.
 - The advisor's own commit subjects MUST follow attribution-integrity
   patterns (e.g. `chore(advisor): ` for state-machine transitions,
   `docs(advisor): ` for L14 retro re-applies).
```

Also update the NEVER-WRITES note about the catch-fire being the only non-JSON runtime write (anchor: lines 121–123):

```diff
@@
-- The catch-fire path is the ONLY path that writes a `.md` file under
-  `.claude/auto-state/` — and that file is gitignored runtime state,
-  not a tracked artifact.
+- Under `.claude/auto-state/`, the skill writes only gitignored
+  runtime state: the catch-fire `.md` dump, the `<phase>.digests.jsonl`
+  digest overflow, and `<phase>.spill/*.txt` tool-output spills — none
+  are tracked artifacts. The auto-handover `.md` is the one tracked
+  `.md` the skill emits, and it lives under `.claude/PRPs/handovers/`
+  (advisor-owned, `chore(advisor):` commit), NOT under
+  `.claude/auto-state/`.
```

**B2 — new §"Context-management invariants" block (insert after §"Cadence invariants", i.e. after line 216, before §"Resume semantics" at line 218):**

```diff
@@
    The skill MUST NOT override this without recording wall-clock
    evidence in a future retro that justifies the change.
 
+## Context-management invariants
+
+These four invariants protect the durable-context substrate (the
+ledger) and the bounded-context discipline. Schema-v2 (see
+`.claude/PRPs/templates/auto-phase-state.template.json`).
+
+1. **Stage-digest ring is data-only.** On each stage transition the
+   skill appends one digest to `stage_digests` AFTER updating `stage`,
+   then trims to the last `DIGEST_RING_MAX` (12), spilling the oldest
+   to `digest_overflow_path`. The ring MUST NOT gate, route, or change
+   cadence — it is a resume/`/compact` read source, nothing more. A
+   change that makes a routing decision depend on a digest is a
+   regression.
+
+2. **Uniform spill guard, not per-tool memory.** Any tool result over
+   `SPILL_THRESHOLD_CHARS` (16000) is written to `spill_dir` and
+   reduced to head+tail+path in context. This generalises the existing
+   per-tool mitigations (`pmd-search-strategy.md` broad-search spill,
+   DQ `log_slice` cap, `cargo-output-capture`); it does not replace the
+   subagent-isolation advice for broad PMD queries — prefer the
+   subagent when a large result is expected, spill when it is not.
+
+3. **Auto-handover refresh is belt-and-braces, not the only handover.**
+   The skill refreshes `last_handover_path` on each transition so the
+   pre-compact discipline is satisfied without hand-authoring. A
+   session may still author a richer hand-written handover; the
+   auto-handover is the floor, not the ceiling. It MUST be committed
+   with a `chore(advisor):` subject (attribution integrity).
+
+4. **Durable context is wide; delegation packets are narrow.** The
+   ledger (advisor-durable) carries phase/stage/cohort/gates/digests/
+   handover/verification/next-action across boundaries. Junior briefs
+   stay narrow and brief-scoped (one task brief, scope, write
+   boundaries, validation expectations, relevant error history) per
+   `advisor-orchestrator.md` §2.2 — the digest ring MUST NOT leak into
+   brief bodies. Symmetry is deliberate: wide where it survives
+   restarts, narrow where it is dispatched.
+
 ## Resume semantics
```

**B3 — extend §"What survives across session boundaries" (anchor: the bullet list at lines 339–355; insert after the `user_gate_history` bullet at line 355):**

```diff
@@
 - `user_gate_history` — what the user has decided so far this phase
   (used by retro author).
+- `stage_digests` — bounded ring of per-transition digests (schema-v2);
+  the self-contained "here's what was happening" narrative Phase 0.5
+  Step E reads to build the COMPACT resume report without conversation
+  context. Overflow beyond the cap lives in `digest_overflow_path`.
+- `last_handover_path` + `last_handover_at` — pointer to the
+  auto-emitted handover refreshed at the last transition; the resume
+  report and `/compact` (compact-phase.md priority 1) cite it as the
+  authoritative active-thread source.
```

### Diff C — `.claude/refs/compact-prompt-approach.md` (Phase 3 only)

Append an optimisation-history entry (anchor: end of the "## Optimisation history" list, after line 107) recording the digest-first `/compact` wiring. (Required by the doc's own rule that editing `compact-phase.md` updates this note.)

```diff
--- a/.claude/refs/compact-prompt-approach.md
+++ b/.claude/refs/compact-prompt-approach.md
@@
   `feedback_thin_wakeup_prompts_verify_live_state.md` (compaction-resume is the same family as
   ScheduleWakeup-resume: a resume-context written before the awaited work resolves).
+- **<DATE OF APPLY>** — Digest-first `/compact` wiring (auto-phase context-management Phase 3).
+  Priority 1 ("active thread") now instructs the summary to read the `/auto-phase` ledger
+  (`.claude/auto-state/<phase>.json` `stage_digests[-1]` + `last_handover_path`) and cite it as
+  the authoritative active-thread state, instead of reconstructing branch/stage/in-flight from
+  conversation. Rationale: the ledger is the one artifact that survives compaction by design, so
+  it is a higher-fidelity active-thread source than the summary's own reconstruction. The
+  next-action carried from the digest stays marked a hypothesis-to-re-verify (unchanged from the
+  2026-05-30 fix). Applies only when the active thread is a `/auto-phase` run with a live ledger;
+  non-auto-phase sessions fall back to the existing priority-1 wording.
```

---

## 7. User-scope command patch notes (files outside the repo)

`~/.claude/commands/auto-phase.md` and `~/.claude/commands/compact-phase.md` are **not in the repo snapshot** (user-scope). Per constraint #7 I do **not** invent exact diffs. Below are insertion points (described against the structure documented in `.claude/refs/auto-phase.md` and `.claude/refs/compact-prompt-approach.md`) and exact text blocks to insert.

### `~/.claude/commands/auto-phase.md`

The spec (`.claude/refs/auto-phase.md`) describes the body's structure: **Phase 0 Step 0** (read the refs file), **Phase 0.5** (session resume, Steps A–E), **Phase 1** (tick procedure), **Phase 2** (stage routing table). Insert the following.

**Block 1 — Phase 1 tick, per-transition digest write (insert at the point where the tick already updates `stage` and writes the ledger):**

```
On every stage transition, AFTER updating `stage` and BEFORE the ScheduleWakeup:
1. Build a digest object:
   {
     "stage": "<new stage>",
     "entered_at": "<UTC iso>",
     "outcome": "<=1 line: what the prior stage produced or decided>",
     "key_ids": { "cohort": <n|null>, "junior": [...], "dq": [...], "pr": <n|null> },
     "next_action_hypothesis": "<=1 line: what the next tick intends (re-verify on resume)>"
   }
2. Append to auto_state.stage_digests.
3. If len(stage_digests) > 12: append the oldest overflow entries (one JSON object per line)
   to auto_state.digest_overflow_path (default .claude/auto-state/<phase>.digests.jsonl),
   then keep only the last 12 in stage_digests.
4. Write the ledger. (Data-only — never gate or route on a digest.)
```

**Block 2 — spill-guard wrapper (insert near the top of Phase 1, as a standing rule applied to all tool results during the tick):**

```
Tool-output spill guard: when any tool result exceeds 16000 chars, write the full result to
auto_state.spill_dir/<stage>-<tool>-<UTC-iso>.txt (default .claude/auto-state/<phase>.spill/),
and retain only the first ~40 and last ~40 lines plus the spill path in context. For broad PMD
semantic queries, prefer running inside a subagent (per pmd-search-strategy.md) so the dump never
enters main context; spill is the fallback when a large result was not anticipated.
```

**Block 3 — auto-handover refresh (insert in Phase 1, immediately after Block 1's digest write):**

```
Auto-handover refresh: after the digest append, write/overwrite
.claude/PRPs/handovers/<phase>-auto-<UTC-date>.md with: current sub-phase, stage, last commit on
the relevant branch (SHA + subject), next concrete action (the digest's next_action_hypothesis,
marked "re-verify on resume"), DQ pending ids, and a one-line concurrent-activity note. Set
auto_state.last_handover_path + last_handover_at. Commit on governance-v0 with subject
"chore(advisor): auto-phase handover refresh <phase> <stage>". This is the floor that satisfies
the advisor-orchestrator.md §1 pre-compact handover discipline automatically.
```

**Block 5 — Phase 0.5 Step E digest-first resume (insert in Step E, where the COMPACT resume report is built):**

```
Build the COMPACT resume report (<=15 lines, <=500 tokens) primarily from the ledger, not from
conversation: read stage_digests[-3:] for the recent narrative and last_handover_path for the
self-contained handover. Surface the latest next_action_hypothesis as a HYPOTHESIS to re-verify
against live TaskList/DQ/PR state before any action (per feedback_thin_wakeup_prompts_verify_
live_state.md). Keep lazy-load + Steps-B-D subagent-delegation discipline unchanged; the digests
are additional durable signal, not a new preload.
```

> Note: there is no "Block 4" in this file — Block 4 belongs to `compact-phase.md` below. Blocks are numbered globally across §7 for cross-reference from §3–§5.

### `~/.claude/commands/compact-phase.md`

The doc (`compact-prompt-approach.md`) describes the prompt's three priorities (active thread / task list verbatim / user messages verbatim) and the exclusion clause. Insert into **priority 1**.

**Block 4 — digest-first active-thread source (insert into the priority-1 "active thread" instruction):**

```
If the active thread is a /auto-phase run (a .claude/auto-state/<phase>.json ledger exists for the
phase named in the session), treat the ledger as the authoritative active-thread source: cite
stage_digests[-1] (stage, outcome, next_action_hypothesis) and last_handover_path verbatim,
rather than reconstructing branch/stage/in-flight state from conversation. The ledger survives
compaction by design; the conversation does not. Keep the next-action marked a hypothesis to
re-verify on resume. For non-/auto-phase sessions, use the existing priority-1 wording unchanged.
```

After editing `compact-phase.md`, apply Diff C to `.claude/refs/compact-prompt-approach.md` in the same change (the doc's own discipline).

---

## 8. JSON schema / data-shape examples

### `stage_digests` entry (ledger, bounded ring, max 12)

```json
{
  "stage": "impl-cohort-2-validating",
  "entered_at": "2026-05-07T16:12:00Z",
  "outcome": "cohort 2 (tasks 4,5) both complete; validate-pending DQ a3f9c1e2b740-007/-008 raised",
  "key_ids": {
    "cohort": 2,
    "junior": [161, 162],
    "dq": ["a3f9c1e2b740-007", "a3f9c1e2b740-008"],
    "pr": null
  },
  "next_action_hypothesis": "dispatch one serial ci-watcher for the cohort-2 pair; re-verify DQ state first"
}
```

### Digest overflow record (`<phase>.digests.jsonl`, append-only, one object per line)

```jsonl
{"stage":"init","entered_at":"2026-05-07T14:00:00Z","outcome":"fresh init; prereqs passed","key_ids":{"cohort":null,"junior":[],"dq":[],"pr":null},"next_action_hypothesis":"run /brehon-clarify then queue planning"}
{"stage":"planning-running","entered_at":"2026-05-07T14:20:00Z","outcome":"planning Junior #150 dispatched","key_ids":{"cohort":null,"junior":[150],"dq":[],"pr":null},"next_action_hypothesis":"one-shot 1200s sleep; DoD smoke test on completion"}
```

### Spill record (inline in a digest, or one line in `<phase>.spill.jsonl`)

```json
{
  "stage": "impl-cohort-2-validating",
  "tool": "memory_search_hybrid",
  "spilled_at": "2026-05-07T16:13:30Z",
  "bytes": 98213,
  "path": ".claude/auto-state/v1-SL-c-2.spill/impl-cohort-2-validating-memory_search_hybrid-20260507T161330Z.txt",
  "head_tail_kept": true,
  "reason": "exceeded SPILL_THRESHOLD_CHARS=16000"
}
```

### Auto-handover front-matter shape (`<phase>-auto-<date>.md`, tracked)

```markdown
# Auto-handover — v1-SL-c-2 — 2026-05-07

- sub_phase: v1-SL-c-2
- stage: impl-cohort-2-validating
- branch: phase-v1-SL-c-2
- last_commit: 4f2a9c1 "feat(governance): cohort 2 task 5 (#162)"
- next_action (HYPOTHESIS — re-verify): dispatch serial ci-watcher for cohort-2 pair
- dq_pending: [a3f9c1e2b740-007, a3f9c1e2b740-008]
- concurrent_activity: none observed (single lane)
```

---

## 9. Validation checklist

### Without a live `/auto-phase` run (static / dry)

1. **JSON validity:** `python -c "import json,io; json.load(io.open('.claude/PRPs/templates/auto-phase-state.template.json', encoding='utf-8'))"` after Diff A — confirm the template still parses and `schema_version == 2`.
2. **Backfill simulation:** write a throwaway v1 ledger (no `stage_digests`), run the Phase 0.5 Step A backfill logic against it, confirm `stage_digests: []` + null path fields are added and no v1 field is mutated.
3. **Ring trim unit check:** feed 15 synthetic digests through the append-then-trim logic; confirm exactly 12 remain in the array and 3 are appended to the JSONL overflow, in order.
4. **Spill threshold check:** feed a 20000-char and a 5000-char synthetic tool result; confirm only the first spills and the in-context residue is head+tail+path.
5. **Gitignore coverage:** `git check-ignore .claude/auto-state/v1-SL-c-2.digests.jsonl .claude/auto-state/v1-SL-c-2.spill/x.txt` — both must report ignored (confirms no `.gitignore` change is needed).
6. **Handover path/commit-subject conformance:** confirm the auto-handover path matches `.claude/PRPs/handovers/<phase>-auto-<date>.md` and the proposed commit subject matches `^(chore|docs)\((advisor|decision-queue)\)` (it is `chore(advisor):`).
7. **Doc anchors intact:** `grep '^## ' .claude/refs/auto-phase.md` after Diff B — confirm no existing heading text changed (Pi `.pi/` + skills cite headings by name; moving/renaming a heading breaks the dual-harness contract). The new heading "## Context-management invariants" is additive.

### During a live `/auto-phase` run

8. **First transition:** after `init` → next stage, confirm `stage_digests` has exactly one entry with a sane outcome + next_action_hypothesis, and `last_handover_path` is set + the handover file exists and parses.
9. **Ring overflow live:** run far enough that >12 transitions occur; confirm the JSONL overflow grows and the array stays capped at 12.
10. **Spill live:** trigger a broad PMD query (or any >16K result); confirm a spill file appears under `<phase>.spill/` and the main context kept only head+tail+path.
11. **Resume test (the load-bearing one):** `/compact`, then re-invoke `/auto-phase <phase>` from a cold session. Confirm Phase 0.5 Step E builds the COMPACT report from `stage_digests` + `last_handover_path`, the report is ≤15 lines, and the next-action is marked a hypothesis. Confirm read-only-until-'continue' (invariant B) still holds.
12. **`/compact` duplication test** (per `compact-prompt-approach.md` empirical loop): run `/context` before `/compact`; after, confirm the summary cites the ledger digest for the active thread and does NOT restate auto-loaded rules.
13. **No-regression on gates/cadence:** confirm the six user gates still fire, catch-fire still terminal, cohort barrier still atomic, and no 300s sleeps appear in the ScheduleWakeup cadence.

---

## 10. Rollback plan

Each phase is independently revertible; the `schema_version` bump is the safety pin.

- **Phase 1 (digest ring):** revert Diff A + Diff B1/B3 + remove §7 Block 1. Old ledgers with `schema_version: 2` still parse under a reverted (v1-only) skill because the extra fields are ignored on read; to be fully clean, set `schema_version` back to 1 in the template only (live ledgers are gitignored and per-phase — delete the stale `<phase>.json` + `.digests.jsonl` to force fresh init, or hand-trim the new fields). The digest ring is inert, so leaving the data in a live ledger is harmless even if the skill stops writing it.
- **Phase 2 (spill guard + handover):** remove §7 Blocks 2/3 + the B2 invariants #2/#3 + the B1 WRITES bullets for spill/handover. Spill files and auto-handovers already written are gitignored (spill) or a tracked but inert `.md` (handover) — delete or leave; neither affects routing.
- **Phase 3 (`/compact` + resume):** revert §7 Blocks 4/5 + Diff C. Phase 0.5 falls back to its current conversation-based report; the ledger digests simply go unread. No data migration needed.
- **Full rollback:** revert all diffs, set `schema_version` to 1 in the template, delete any live `<phase>.json`/`.digests.jsonl`/`.spill/` to force a clean v1 init on next run. Because all runtime state is gitignored and per-phase, no shared state is corrupted by a rollback.

---

## 11. Risks and safeguards

| Risk | Likelihood | Safeguard |
|---|---|---|
| Digest write adds latency / token cost on every transition | Low | Digest is ≤5 lines; written in the same ledger write that already happens each transition. No extra tool call. |
| Ring/overflow logic loses the oldest narrative on a long phase | Low | Overflow JSONL is append-only and retains everything beyond the cap; resume reads the ring, debug reads the JSONL. Nothing is dropped, only moved. |
| Spill threshold mis-set → either spills too eagerly (loses signal) or too late (context blowup) | Medium | Threshold is a single tunable constant (16000); validate via checklist #4/#10. Conservative default leans toward keeping head+tail so signal is never fully lost. |
| Auto-handover commit pollutes `governance-v0` history with churn | Medium | `chore(advisor):` subject keeps it classifiable + filterable; one refresh per transition (not per tick). If churn is a concern, gate the commit to "only when stage actually changed" (already the trigger) and squash-free per existing PR discipline. |
| Auto-handover races a concurrent canonical session writing `governance-v0` | Low | The skill is the advisor session for this phase; the multi-lane discipline (`multi-lane-worktree.md`) already governs who writes `governance-v0`. Auto-handover follows the same atomic read-mutate-commit pattern as other advisor commits. |
| `schema_version` bump breaks a session running the OLD skill against a NEW ledger | Low | Additive forward-only fields; the old skill ignores unknown keys (same property the v3 DQ migration relies on). The pin is in the template, not the live ledger. |
| Renaming/moving a heading in `auto-phase.md` breaks Pi/skill citations | Low | Diff B only **adds** a heading ("## Context-management invariants") and **appends** to existing lists; no existing heading text changes. Checklist #7 verifies. |
| Digest ring tempts a future change to route on its contents (invariant erosion) | Medium | Invariant #1 (B2) states data-only explicitly; retro flags any routing-on-digest change as a regression. |
| Spill guard interacts badly with the existing PMD subagent-isolation advice | Low | Invariant #2 (B2) keeps subagent-isolation as the *preferred* path for anticipated-large queries; spill is the fallback for the unanticipated case. They compose, not conflict. |

---

*End of patch plan. No repository files were modified and no patches were applied in producing this document.*
