# Brehon Context-Management Review — `/auto-phase` Multi-Agent Advisory Workflow

**Repository:** `/home/user/workspace/lemmy-local` (Lemmy 1.0-beta fork → `barrie-cork/lemmy`, branch `governance-v0`)
**Scope:** REVIEW ONLY — no repository files modified, no commits created. This document is the sole deliverable.
**Date:** 2026-06-11
**Reviewer focus:** context-management opportunities for the `/auto-phase` advisor↔junior multi-agent delegation loop.

---

## 1. Executive summary

The repository runs a **dual-harness** setup. Pi sessions (the **advisor**) load `AGENTS.md` + `.pi/PROJECT_CONTEXT.md` and execute the TypeScript extension `.pi/extensions/lemmy-hooks.ts`. Claude Code sessions load `CLAUDE.md` + `.claude/**`. The two harnesses share the on-disk `.claude/` state directory but **not** the same hook runtime.

The central finding: **the `/auto-phase` deterministic state ledger already exists** — its schema is fully specified in `.claude/PRPs/templates/auto-phase-state.template.json` (a 29-stage state machine with cohort tracking, gate history, and per-member error-class history), and the runtime file lives at `.claude/auto-state/<phase>.json` (gitignored, created on first `/auto-phase` run; absent at review time because no phase is mid-flight). **The Pi extension has zero awareness of it** — `grep` across `.pi/**` (excluding npm) returns no references to `auto-state`, `auto-phase`, or `autophase`.

The only deterministic-state injection the Pi extension performs today is `coordinationStateSummary()` (lemmy-hooks.ts), which reads `.claude/decision-queue.json` (pending) and `.claude/task-hopper.json` (in-progress/escalated) and emits a one-line summary. It does **not** read the auto-state ledger, does **not** distinguish advisor vs. junior injection, and the `tool_result` handler does **not** compress or truncate large results.

**Recommendation:** do **not** invent a new schema or a second harness. Instead, apply **incremental Pi-side patches** to `lemmy-hooks.ts` plus a small new state-helper module under `.pi/`, so the advisor session reads, injects, and maintains the *existing* `.claude/auto-state/<phase>.json` ledger. Then layer in (a) phase-end summarization and (b) a compressed tool-result logging wrapper. The three implementation targets map onto:

| Target | Status today | Recommended action |
|---|---|---|
| (a) structured state maintenance | **partial** — DQ + hopper only, no auto-state | bridge the existing ledger into `before_agent_start` |
| (b) phase-end summarization | **absent** | add a `/brehon-phase-summary` command + stage-transition digest writer |
| (c) compressed tool-result logging | **absent** — `tool_result` never truncates | add a result-size guard + JSONL spill in `tool_result` |

---

## 2. Repository / harness inventory

### 2.1 Agent-control / harness files (verified present)

| Path | Role | Lines |
|---|---|---|
| `AGENTS.md` | Pi entry point — advisor persona + four-role model + ownership boundaries | 36 |
| `CLAUDE.md` | Claude Code entry point — ADR hard constraints, four-role YAML, six user gates, polling loop, canonical paths | (read in full) |
| `.pi/PROJECT_CONTEXT.md` | Pi substantive context — Brehon constraints, dual-harness boundary, Rust quick-reference, role table, setup decisions | 257 |
| `.pi/settings.json` | Pi config — `defaultProvider: openai-codex`, `defaultModel: gpt-5.5`, skills/extensions paths, **compaction (`keepRecentTokens 40000`, `reserveTokens 32000`)** | — |
| `.pi/extensions/lemmy-hooks.ts` | **THE core integration file** — modes, context injection, path policy, guards, telemetry | 871 |
| `.pi/hook-scripts/observation-capture.sh` | Shadow-mode PostToolUse JSONL telemetry; secret-redaction; never blocks | — |
| `.pi/skills/{impl-task,ci-debug,...}/SKILL.md` | Pi-side role skills auto-injected by mode | — |
| `.claude/PRPs/templates/auto-phase-state.template.json` | **The existing BrehonState schema** (29-stage machine) | 86 |
| `.claude/refs/auto-phase.md` | `/auto-phase` state-machine spec (lazy-loaded by skill body) | — |
| `.claude/rules/*.md` | Always-load rule corpus (~35% of 200K budget per harness-audit) | many |
| `~/.claude/commands/auto-phase.md` | The `/auto-phase` skill body (user-scope) | — |

### 2.2 State files (runtime, gitignored)

| Path | Writer | Read by |
|---|---|---|
| `.claude/decision-queue.json` | advisor + junior subagents | Pi `coordinationStateSummary()`, advisor polling loop |
| `.claude/task-hopper.json` | advisor | Pi `coordinationStateSummary()` |
| `.claude/auto-state/<phase>.json` | `/auto-phase` skill (Claude side) | `/auto-phase` skill only — **NOT Pi** |
| `.claude/.dq-session-id` | DQ id helper | DQ scripts |
| `.claude/agent-activity.json` | SessionStart/End hooks | session-awareness checks |
| `~/.cache/tw-observations/${PPID}.jsonl` | `observation-capture.sh` | passive audit |

---

## 3. Current context-management behavior observed (with file locations)

All references below are to `.pi/extensions/lemmy-hooks.ts` unless noted.

### 3.1 Context injection (`before_agent_start`)

The extension appends, in order, to `event.systemPrompt`:

1. **Sliced Pi Project Context** — only the headings named in `MODE_CONTEXT[mode].projectContextHeadings` are spliced out of `.pi/PROJECT_CONTEXT.md` (progressive disclosure by mode).
2. **Active Brehon Pi Mode** block — persona + authorized read paths for the active mode.
3. **Pre-Phase Audit Reminder** (`prePhaseReminder`, module-scope).
4. **Coordination State** — the output of `coordinationStateSummary()`.
5. **Auto-injected role skill** — `MODE_CONTEXT[mode].recommendedSkill`.
6. **Filtered rule index** — rule filenames filtered by `MODE_CONTEXT[mode].ruleFilter`.

This is genuine, working progressive-disclosure context injection. **Gap:** step 4 is the only deterministic project-state injection and it knows nothing about phase/stage/cohort.

### 3.2 The one existing state summary (`coordinationStateSummary()`)

Reads `.claude/decision-queue.json` → counts `pending`; reads `.claude/task-hopper.json` → counts `in_progress` / `escalated`; emits a one-line summary. **Does not read `.claude/auto-state/<phase>.json`.** This is the seam to extend.

### 3.3 Seven Brehon modes (`BrehonMode`)

`main-safe | planning | impl-task | review-readonly | bm | ci-debug | harness-maintenance`. Each has `persona`, `projectContextHeadings`, `ruleFilter`, `recommendedSkill`, `authorizedReadPaths` (in `MODE_CONTEXT`) and `label`/`description`/`instructions` (in `BREHON_MODES`). Default mode is `main-safe`.

### 3.4 Guards (`tool_call`)

`pathPolicyDecision(mode, toolName, filePath)` enforces per-mode write boundaries + "Rust requires a plan file" + the `.claude/` boundary; override via `/brehon-override` (one-shot `overridePaths`). `tool_call` also blocks raw cargo (`rawCargoPattern`), secret reads/shells, a bash firewall (`BASH_BLOCKLIST`), and worktree drift.

### 3.5 `tool_result` handler

Runs `observation-capture.sh`; tracks `editsSinceRead` (threshold 5 → nudge); runs lesson-frontmatter + lesson-PMD-sync hooks. **Auto-commit is intentionally disabled** (confirmed in code + PROJECT_CONTEXT setup-decisions log). **It does NOT truncate or compress large tool results** — this is the gap for implementation target (c).

### 3.6 `session_shutdown`

Warn-only retro nudge via `retro-check.sh`. No state digest written.

### 3.7 Pi-native compaction

`.pi/settings.json` enables Pi's built-in conversation compaction (`keepRecentTokens 40000`, `reserveTokens 32000`). This handles *conversational* context but is blind to *deterministic project state* — exactly the transient-vs-durable distinction this review turns on.

---

## 4. Extracted roles / modes / invariants / gates / scopes / workflow phases

### 4.1 Four-role model (CLAUDE.md + AGENTS.md + PROJECT_CONTEXT.md)

| Role | Runs on | Model | Authors content? | Task prefix |
|---|---|---|---|---|
| **advisor** | laptop (Pi session, this harness) | opus-4-7 | **NO** — meta-oversight only | none |
| planning | EliteDesk/Junior | opus-4-8 | yes (plans) | `[role:planning]` |
| impl | EliteDesk/Junior | sonnet-4-6 | yes (Rust) | `[role:impl-task]` |
| bm | EliteDesk/Junior | haiku-4-5 | git/PR topology | `[role:bm-task]` |
| ci_watcher | EliteDesk/Junior | haiku-4-5 | mutates DQ validate-pending | `[role:ci-watcher]` |

The advisor **never authors plans, Rust, migrations, or PRs**. It briefs, queues, polls, triages DQ, runs gate checks, and harvests retros.

### 4.2 Six mandatory user gates (never auto-skipped)

1. Plan approval (after DoD smoke test + watchpoint-specificity gate)
2. Judgment-heavy DQ (ADR-affecting / scope-changing / visible-to-others)
3. CR triage approval (four-bucket counts)
4. Phase-2 e2e local-vs-dispatch (never auto-picked post-PR-#105)
5. Merge confirm
6. Retro sign-off

### 4.3 Hard invariants relevant to state tracking

- **15 ADRs append-only**; v0 scope = exactly 11 endpoints. A plan contradicting an ADR → STOP/surface, never silent-fix.
- **ADR-015 pseudonymity** is load-bearing on governance write paths (callsite `actor_pseudonym::get_or_create`).
- **Catch-fire is a terminal state** (`stage: "catch-fire"`); the loop halts and surfaces.
- **Cohort barriers are atomic** — all members reach `complete`+validated before advancing.
- **Cycle-count meta-rule:** ≥3 fails with same `(error_class, file_basename)` → hard catch-fire regardless of allowlist.
- **DQ attribution integrity:** `answered_by: "advisor"` only from advisor-scoped commits; `approved_by` advisor-exclusive.
- **No cargo on the daemon** — validate-pending-laptop pattern.
- **Cross-lane total cap = 2 running Junior tasks** (shared `.git/index.lock`).

### 4.4 Workflow phases (the `/auto-phase` stage machine)

29 stages from `auto-phase-state.template.json`:
`init → bm-cut-{running,done} → planning-{running,revise,approved-pending-user} → impl-cohort-N{,-running,-validating} → impl-fix-N-running → phase-2-e2e-N{,-running} → bm-pr-{pending,running} → cr-wait → bm-poll-cr-running → bm-triage-running → cr-triage-pending-user → fix-in-pr-cohort-N{,-running} → verify-running → merge-{pending-user,executing} → retro-{author,pending-user} → phase-transition → done | catch-fire`.

### 4.5 Pi mode → workflow-phase mapping (the injection key)

The Pi `BrehonMode` is the natural index for *what to inject*. The mapping is not currently materialized but is implicit:

| Pi mode | Active during stages |
|---|---|
| `main-safe` | between phases / idle polling |
| `planning` | `planning-*` |
| `impl-task` | `impl-cohort-*`, `impl-fix-*`, `fix-in-pr-cohort-*` |
| `bm` | `bm-cut-*`, `bm-pr-*`, `bm-poll-cr-*`, `bm-triage-*`, `merge-*` |
| `ci-debug` | `phase-2-e2e-*`, validate-fail triage |
| `review-readonly` | `verify-running`, retro authoring |
| `harness-maintenance` | meta-edits |

---

## 5. `/auto-phase` implications for advisor↔junior multi-agent runs

`/auto-phase` is **not** a single coding session — it is a many-hour orchestration in which the advisor (Pi) cycles `poll → detect transition → triage DQ → brief → dispatch Junior → await → gate → repeat`, spanning multiple `/compact` boundaries and session restarts. Context management therefore has two distinct consumers with **opposite** needs:

### 5.1 Advisor context injection (wide, durable, resume-oriented)

The advisor needs the **full BrehonState digest** to resume coherently after any boundary:
- current `phase` + `stage` (where in the 29-stage machine)
- `current_cohort` (n, members, each member's `junior_id` / `result` / `error_class_history`)
- `last_known_phase_tip` (SHA — detect daemon-side merges that completed while the session was down)
- `user_gate_history` (which gates already cleared — never re-ask)
- `last_dq_pending_count` / `last_dq_pending_ids` (surface-first ritual)
- `phase_2_e2e_mode` (so gate 4 isn't re-prompted)
- `catch_fire` (if set, the loop is terminal)

This digest is **what the advisor must never lose across compaction**. It is precisely the existing `.claude/auto-state/<phase>.json` schema. The advisor's `before_agent_start` should inject a *compact rendering* of this file (≤15 lines / ≤500 tokens, matching the existing compact-report discipline in `auto-phase.md`).

### 5.2 Junior / task-specific context injection (narrow, scoped, single-task)

A Junior worker (planning/impl/bm/ci-watcher) runs `claude -p` on the daemon in an isolated worktree. It must receive a **scoped delegation packet**, *not* the advisor's full ledger:
- the **single** brief path (`.claude/PRPs/briefs/<phase>-<role>-<n>.md`)
- the **one** task scope + file boundaries
- the mode's authorized read paths + write boundaries
- the relevant mandatory file-class lessons (§2.4 injection table)
- the `validate-pending` DQ id it must write (impl) or mutate (ci-watcher)
- **NOT** the gate history, cohort roster, or cross-phase state — those are advisor-only and would bloat the worker's context and leak coordination concerns into an execution role.

### 5.3 The asymmetry, stated as a design rule

> **Advisor injection = "where are we in the whole phase, and what must I not lose?"**
> **Junior injection = "what is my one job, and what may I touch?"**

The Pi extension already encodes half of this via `MODE_CONTEXT` (mode persona + authorized paths + filtered rules). What's missing is the **state digest** for the advisor side. The junior side is largely handled by the brief + mode skill; the only Pi-side need is to ensure the junior delegation packet is built from the brief, not from the advisor's resident state.

---

## 6. Candidate implementation points table

| # | File / location | Change required | Effort | Risk | Expected value |
|---|---|---|---|---|---|
| 1 | new `.pi/lib/auto-state.ts` (helper module) | `readAutoState(phase)`, `renderAdvisorDigest(state)`, `patchAutoState(phase, patch)` over `.claude/auto-state/<phase>.json` | S | Low (pure read/write of an existing schema) | Foundation for all of below |
| 2 | `lemmy-hooks.ts` `coordinationStateSummary()` | extend to call `renderAdvisorDigest()` when an auto-state file exists for the active phase | S | Low (additive; null-safe when file absent) | Advisor resumes with phase/stage/cohort/gates in-prompt |
| 3 | `lemmy-hooks.ts` `before_agent_start` | gate the digest injection on **advisor context only** (mode ∈ advisor set); juniors get the existing scoped block | S | Low | Enforces the §5 asymmetry |
| 4 | `lemmy-hooks.ts` `tool_result` | size-guard: if a tool result exceeds N chars, spill full text to `.claude/auto-state/tool-results/<ts>.txt` and replace the in-prompt copy with a head/tail + path | M | Med (must not break observation-capture or lesson hooks; must preserve existing return shape) | Caps context bloat on long impl/CR logs |
| 5 | new `~/.claude/commands/brehon-phase-summary.md` (Claude side) + optional Pi `/brehon-phase-summary` registerCommand | at stage transitions, write a ≤500-token digest of the closing stage into the auto-state file's `last_action` + a `stage_summaries[]` ring | M | Low (write to existing gitignored runtime file) | Phase-end summarization; cheap resume |
| 6 | `lemmy-hooks.ts` `session_shutdown` | before the retro nudge, flush a final advisor digest line to the auto-state file (`last_session_ended_at`, `last_known_phase_tip`) | S | Low | Clean-resume signal for the next session |
| 7 | `.pi/PROJECT_CONTEXT.md` (1 heading) | document the advisor-vs-junior injection asymmetry so the rule is discoverable | S | None | Keeps the dual-harness contract legible |

(Effort: S ≈ <1 day, M ≈ 1–2 days. Risk relative to the live orchestration loop.)

---

## 7. Proposed `BrehonState` TypeScript interface + supporting types

This mirrors `auto-phase-state.template.json` exactly (so the Pi extension and the Claude skill share one on-disk contract) and adds the advisor/junior-injection-relevant typing. No new fields are invented beyond what the template already carries, except the optional additive `stage_summaries[]` ring (target #5).

```typescript
// .pi/lib/auto-state.ts

/** 29-stage machine — verbatim from auto-phase-state.template.json `_stage_enum`. */
export type BrehonStage =
  | "init"
  | "bm-cut-running" | "bm-cut-done"
  | "planning-running" | "planning-revise" | "planning-approved-pending-user"
  | "impl-cohort-N" | "impl-cohort-N-running" | "impl-cohort-N-validating"
  | "impl-fix-N-running"
  | "phase-2-e2e-N" | "phase-2-e2e-N-running"
  | "bm-pr-pending" | "bm-pr-running"
  | "cr-wait" | "bm-poll-cr-running" | "bm-triage-running" | "cr-triage-pending-user"
  | "fix-in-pr-cohort-N" | "fix-in-pr-cohort-N-running"
  | "verify-running"
  | "merge-pending-user" | "merge-executing"
  | "retro-author" | "retro-pending-user"
  | "phase-transition"
  | "done" | "catch-fire";

export type Phase2E2eMode = "local" | "dispatch" | null;

/** Pi-side mirror of the seven Brehon modes; advisor vs junior is the injection axis. */
export type BrehonMode =
  | "main-safe" | "planning" | "impl-task"
  | "review-readonly" | "bm" | "ci-debug" | "harness-maintenance";

/** Modes that run in the advisor (Pi) session — get the full state digest. */
export const ADVISOR_MODES: ReadonlySet<BrehonMode> =
  new Set(["main-safe", "review-readonly", "harness-maintenance", "ci-debug"]);
/** Modes that build a scoped delegation packet for a Junior worker. */
export const JUNIOR_DELEGATION_MODES: ReadonlySet<BrehonMode> =
  new Set(["planning", "impl-task", "bm"]);

export interface ErrorClassEntry {
  workflow_run_id: number | null;
  error_class: string;      // e.g. "E0277"
  file: string;             // basename only — line numbers drift
  ts: string;               // ISO 8601
}

export interface CohortMember {
  task_id: number;
  junior_id: number | null;
  brief_path: string | null;
  validate_pending_dq_id: string | null;   // composite v3 id, e.g. "a1b2c3d4e5f6-001"
  ci_watcher_junior_id: number | null;
  result: "pass" | "fail" | "cancelled" | "timed_out" | null;
  error_class_history: ErrorClassEntry[];
}

export interface CurrentCohort {
  n: number;
  members: CohortMember[];
  started_at: string;
}

export type UserGate =
  | "plan-approval" | "judgment-dq" | "cr-triage"
  | "phase-2-e2e" | "merge-confirm" | "retro-signoff";

export interface UserGateDecision {
  gate: UserGate;
  decided_at: string;
  decision: "approve" | "reject" | "defer" | string;
  notes: string | null;
}

export interface CatchFire {
  stage_when_fired: BrehonStage;
  reason: string;
  log_slice_path: string;
  fired_at: string;
}

/** Optional ring of stage-close digests — the §6 #5 phase-end-summarization add. */
export interface StageSummary {
  stage: BrehonStage;
  closed_at: string;
  digest: string;           // ≤500 tokens
}

/** The on-disk ledger. Superset-compatible with auto-phase-state.template.json. */
export interface BrehonState {
  schema_version: number;                  // currently 1
  phase: string;                           // e.g. "v1-SL-c-2"
  started_at: string;
  session_id: string;                      // 12-hex per advisor session
  last_session_ended_at: string | null;
  resume_count: number;
  last_known_phase_tip: string | null;     // SHA of last-observed phase-branch tip
  stage: BrehonStage;
  current_cohort: CurrentCohort | null;
  phase_2_e2e_mode: Phase2E2eMode;
  phase_2_e2e_mode_chosen_at: string | null;
  junior_tasks: Record<string, number | null>;  // stage → most-recent Junior task id
  last_action: string | null;
  last_action_at: string | null;
  last_poll_at: string | null;
  last_fetch_at: string | null;
  last_dq_pending_count: number;
  last_dq_pending_ids: string[];
  user_gate_history: UserGateDecision[];
  push_grants_used: number;
  catch_fire: CatchFire | null;
  stage_summaries?: StageSummary[];        // NEW (additive, optional) — §6 #5
}

/** What the advisor's before_agent_start injects (wide, durable). */
export interface AdvisorDigest {
  phase: string;
  stage: BrehonStage;
  cohort: { n: number; members: { task_id: number; junior_id: number | null; result: string | null }[] } | null;
  pendingDqIds: string[];
  gatesCleared: UserGate[];
  phase2Mode: Phase2E2eMode;
  phaseTip: string | null;
  catchFire: CatchFire | null;
}

/** What a Junior worker receives (narrow, scoped). Built from the brief — NOT from BrehonState. */
export interface JuniorDelegationPacket {
  role: "planning" | "impl-task" | "bm-task" | "ci-watcher";
  briefPath: string;
  taskScope: string;
  authorizedWritePaths: string[];
  mandatoryLessons: string[];              // from the §2.4 file-class injection table
  validatePendingDqId: string | null;     // impl writes / ci-watcher mutates
}
```

---

## 8. Draft design — state-maintenance hooks

**New module `.pi/lib/auto-state.ts`** (target #1):

```typescript
import { readFileSync, writeFileSync, existsSync, mkdirSync } from "fs";
import { join } from "path";

const autoStatePath = (phase: string) =>
  join(".claude", "auto-state", `${phase}.json`);

export function readAutoState(phase: string): BrehonState | null {
  const p = autoStatePath(phase);
  if (!existsSync(p)) return null;             // §3 input-validation: absent ≠ error
  try {
    return JSON.parse(readFileSync(p, "utf8")) as BrehonState;
  } catch {
    return null;                               // malformed → null, never throw into the hook
  }
}

export function renderAdvisorDigest(s: BrehonState): string {
  const lines = [
    `## Auto-Phase State (${s.phase})`,
    `Stage: ${s.stage}   Resume #${s.resume_count}   PhaseTip: ${s.last_known_phase_tip ?? "—"}`,
    s.catch_fire
      ? `⛔ CATCH-FIRE @ ${s.catch_fire.stage_when_fired}: ${s.catch_fire.reason}`
      : null,
    s.current_cohort
      ? `Cohort ${s.current_cohort.n}: ` +
        s.current_cohort.members
          .map((m) => `T${m.task_id}=${m.result ?? "running"}` +
            (m.error_class_history.length ? `(${m.error_class_history.length} fails)` : ""))
          .join(", ")
      : `Cohort: none`,
    `Pending DQ: ${s.last_dq_pending_count}` +
      (s.last_dq_pending_ids.length ? ` [${s.last_dq_pending_ids.join(", ")}]` : ""),
    `Gates cleared: ${s.user_gate_history.map((g) => g.gate).join(", ") || "none"}`,
    s.phase_2_e2e_mode ? `Phase-2 e2e mode: ${s.phase_2_e2e_mode}` : null,
  ].filter(Boolean);
  return lines.join("\n");                      // ≤15 lines / ≤500 tokens by construction
}

export function patchAutoState(phase: string, patch: Partial<BrehonState>): void {
  const cur = readAutoState(phase);
  if (!cur) return;                             // never CREATE — the skill owns creation
  const next = { ...cur, ...patch };
  writeFileSync(autoStatePath(phase), JSON.stringify(next, null, 2));
}
```

**Wiring into `lemmy-hooks.ts`:**

- `coordinationStateSummary()` (target #2): after the existing DQ+hopper summary, derive the active phase (from `git branch --show-current` → `phase-v1-<lane>` → `<phase>`, or a module-scope `brehonPhase`), call `readAutoState(phase)`, and append `renderAdvisorDigest()` **only when** the result is non-null.
- `before_agent_start` (target #3): inject the digest block **only when** `ADVISOR_MODES.has(brehonMode)`. For `JUNIOR_DELEGATION_MODES`, leave the existing scoped block (mode persona + authorized paths + filtered rules) and do **not** add the digest — this is the §5 asymmetry, enforced in code.
- `session_shutdown` (target #6): `patchAutoState(phase, { last_session_ended_at: nowIso(), last_known_phase_tip: <current SHA> })` before the retro nudge.

**Invariant preserved:** the Pi extension never *creates* the auto-state file (the `/auto-phase` skill owns creation and stage transitions). Pi only **reads** for injection and **patches** session-boundary fields — so the two harnesses never fight over stage ownership.

---

## 9. Draft design — phase-end summarizer (target #5)

**Goal:** at each stage transition, capture a ≤500-token digest of the closing stage so a resuming session (or the user) reconstructs "what just happened" without re-reading logs.

**Mechanism (Claude-side `/auto-phase` skill writes; Pi-side reads):**

1. When the skill transitions `stage A → stage B`, it appends to `state.stage_summaries[]` (a bounded ring, last ~8 entries):
   ```json
   { "stage": "impl-cohort-2-validating", "closed_at": "<iso>",
     "digest": "Cohort 2 (T4,T5) both pass. DQ a1b2-007 validate-pending→resolved. Next: bm-pr." }
   ```
2. It also sets `last_action` / `last_action_at` (already in the schema).
3. Optional Pi `registerCommand("/brehon-phase-summary")` renders the ring as a one-screen recap on demand — useful when the user returns to a long-running phase.

**Why a ring, not append-forever:** the file is read on every advisor `before_agent_start`; an unbounded log would re-inflate the very context budget this targets. Eight entries covers the typical stage span within a phase; older context lives in the runlog (`.claude/runlog/<phase>-runlog.md`) and retro report.

**Token discipline:** each digest is generated under the existing "compact report ≤15 lines / ≤500 tokens" rule from `auto-phase.md`. This is summarization of *deterministic transitions*, not free-form narration.

---

## 10. Draft design — compressed tool-result logging wrapper (target #4)

**Problem:** `tool_result` in `lemmy-hooks.ts` passes large results (CR logs, cargo output, broad PMD `memory_search_hybrid` dumps — the latter explicitly flagged at 85K–105K chars in `pmd-search-strategy.md`) straight into the conversation. Pi's native compaction eventually trims them, but only after they've already inflated the window.

**Design (additive guard in `tool_result`):**

```typescript
const TOOL_RESULT_MAX = 8000;                 // chars; tune empirically
const SPILL_DIR = join(".claude", "auto-state", "tool-results");

function compressToolResult(name: string, text: string): { text: string; spilled?: string } {
  if (text.length <= TOOL_RESULT_MAX) return { text };
  const stamp = new Date().toISOString().replace(/[:.]/g, "-");
  const path = join(SPILL_DIR, `${stamp}-${name}.txt`);
  mkdirSync(SPILL_DIR, { recursive: true });
  writeFileSync(path, text);                   // full fidelity preserved on disk
  const head = text.slice(0, 3000);
  const tail = text.slice(-1500);
  return {
    text:
      `${head}\n\n…[${text.length - 4500} chars elided — full result at ${path}]…\n\n${tail}`,
    spilled: path,
  };
}
```

**Constraints to honour (so existing behavior is not broken):**
- Run **after** `observation-capture.sh` and the lesson hooks (they consume the raw result; the compression only affects what re-enters the prompt).
- Preserve the handler's existing return shape — replace only the result *text*, never the metadata.
- **Never** compress results the path-policy/guard logic inspects for blocking decisions (those run in `tool_call`, not `tool_result`, so they're unaffected — verify during impl).
- `SPILL_DIR` lives under the already-gitignored `.claude/auto-state/` so spilled logs never get committed.
- Input-validation guard (universal-guards #3): if `text` is empty/binary, skip compression and pass through.

**Value:** caps the per-result context cost during the longest-running stages (impl validate, CR triage) — exactly the stages where the advisor loop accumulates the most tool output across a multi-hour run.

---

## 11. Phased implementation plan (smallest safe changes first)

Each phase is independently shippable and reversible. All touch only `.pi/**` + new gitignored runtime paths + one PROJECT_CONTEXT heading — no `crates/`, `migrations/`, or `tests/`, so per `phase-branch.md` this is **direct-on-`governance-v0` meta-work** (CR review would be net-noise on TypeScript harness prose), *except* if `.pi/` changes are bundled with anything CR reviews well — then PR.

| Phase | Change | Files | Validates by |
|---|---|---|---|
| **P1** | Read-only digest injection | new `.pi/lib/auto-state.ts` (`readAutoState`, `renderAdvisorDigest`); extend `coordinationStateSummary()` | Start an advisor session with a hand-placed `.claude/auto-state/<phase>.json` fixture; confirm the digest appears in the system prompt and is null-safe when absent |
| **P2** | Advisor/junior asymmetry gate | `before_agent_start` mode gate (ADVISOR vs JUNIOR sets) | Confirm `impl-task` mode does NOT get the digest; `main-safe` does |
| **P3** | Session-boundary patch | `patchAutoState`; `session_shutdown` flush | Confirm `last_session_ended_at` + tip written on clean shutdown |
| **P4** | Tool-result compression | `tool_result` size-guard + spill | Confirm a >8K result is head/tail-trimmed in-prompt and full text lands in the spill dir; confirm observation-capture + lesson hooks still see raw text |
| **P5** | Phase-end summaries | `/auto-phase` skill writes `stage_summaries[]` ring (Claude side); optional Pi `/brehon-phase-summary` | Confirm a stage transition appends one digest ≤500 tokens; ring caps at 8 |
| **P6** | Document the asymmetry | one heading in `.pi/PROJECT_CONTEXT.md` | Read-back |

**Sequencing rationale:** P1–P3 are pure additive reads/patches of an existing schema — lowest risk, highest immediate value for resume coherence. P4 carries the only medium risk (must not perturb the hook pipeline) and is isolated last among the code changes. P5 spans the harness boundary (skill writes, Pi reads) so it lands after the read path (P1) is proven. P6 is documentation.

---

## 12. Evaluation against the stated criteria

| Criterion | How the design meets it |
|---|---|
| **Reduce context bloat** | Target #4 caps oversized tool results at the `tool_result` seam (the 85–105K PMD dump is the canonical offender); target #5's bounded ring prevents the state file itself from re-inflating the window; the advisor digest is ≤500 tokens by construction. |
| **Preserve invariants/scope across long sessions** | The advisor digest surfaces `stage`, `user_gate_history`, `catch_fire`, and `last_known_phase_tip` on every `before_agent_start` — so a post-compaction session never re-asks a cleared gate, never misses a catch-fire, and detects daemon-side merges. ADR/scope invariants stay in the always-load rule corpus (unchanged). |
| **Fit Pi's extension model** | Everything routes through existing hooks (`before_agent_start`, `tool_result`, `session_shutdown`) and one helper module — no second harness, no new runtime. The advisor/junior split reuses the existing `MODE_CONTEXT` axis. |
| **Auditable / practical for a solo Markdown workflow** | State stays in the existing gitignored JSON ledger + plain-text spill files; phase summaries are human-readable digests; nothing requires a DB or service. The `/brehon-phase-summary` recap is a one-screen Markdown render. |

**Transient vs. deterministic separation (the core principle):** Pi's native compaction (`.pi/settings.json`) handles *transient conversational* context. The auto-state ledger holds *deterministic project state*. This design keeps them separate — the extension never tries to summarize conversation (Pi does that) and Pi's compaction never has to reconstruct phase/stage (the ledger holds it). That separation is the durable win.

---

## 13. Open questions / assumptions

1. **Active-phase derivation.** The Pi extension must know the current `<phase>` to locate `.claude/auto-state/<phase>.json`. Assumption: derive from `git branch --show-current` (`phase-v1-<lane>` → `<phase>`) or a module-scope `brehonPhase` set on mode switch. In **Mode B (mobile)** the canonical checkout sits on `governance-v0` while the phase runs on the daemon — the extension would need the phase name passed explicitly (e.g. via the `/auto-phase` invocation or an env var). **Needs confirmation.**
2. **`ADVISOR_MODES` membership of `ci-debug`.** `ci-debug` runs in the advisor session (validate-fail triage) but is narrowly scoped. Placed it in `ADVISOR_MODES` so it gets the digest, but if `ci-debug` is ever delegated to a Junior, it must move to the junior set. **Assumption, flag at impl.**
3. **`TOOL_RESULT_MAX` threshold.** 8000 chars is a starting guess; should be tuned against real CR/cargo logs. The Pi compaction `keepRecentTokens` (40000) is the upper bound to stay well under.
4. **Who writes `stage_summaries[]`.** Proposed: the Claude-side `/auto-phase` skill (it owns stage transitions). Pi only reads. If the user prefers Pi to also write transition digests, target #5 moves partly into `tool_result`/command space — slightly higher coupling. **Design choice to confirm.**
5. **PR vs direct-commit for `.pi/` changes.** Per `phase-branch.md`, pure harness meta-work is direct-on-`governance-v0`. Confirmed this is meta-work, but if any phase bundles a `crates/` touch it must go via PR. **Assumption holds only while changes stay inside `.pi/` + runtime paths.**
6. **AppleDouble noise.** The repo carries macOS `._*` metadata files (several `.claude/rules/._*.md` surfaced as garbled binary). Harmless to this design; noted only so an implementer doesn't mistake them for real rule files.

---

## Limitations of this review

- **Static read only.** No `/auto-phase` run was observed live; the stage machine and injection behavior are inferred from the template schema, the skill spec (`auto-phase.md`), and the extension source — not from a running session's emitted prompts.
- **`auto-state/<phase>.json` was absent at review time** (no phase mid-flight), so the digest rendering is validated against the template fixture, not a real runtime file.
- **Pi runtime not executed.** The `@earendil-works/pi-coding-agent` ExtensionAPI hook contracts are read from the extension's usage, not from running the harness; the proposed wiring should be type-checked against the actual `ExtensionAPI` signatures at impl time.
- **No code was modified or committed** (REVIEW ONLY, as instructed).
