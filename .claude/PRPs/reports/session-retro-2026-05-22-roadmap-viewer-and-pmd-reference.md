# Session retro — 2026-05-22 — roadmap viewer + PMD reference

**Harness:** claude-code
**Session window:** 2026-05-22 (afternoon IST) → same day (~30 min)
**Branch at start:** `e9caf8933` (`governance-v0`)
**Branch at end:** `e9caf8933` (`governance-v0`) — no commits this session (viewer is untracked)
**Files touched:** 1 new untracked (v1-roadmap-viewer.html), 1 PMD write (memory #483)
**Commits:** 0 explicit (viewer left untracked per user intent; PMD write via MCP)

## TL;DR

Short session: user asked for an HTML dashboard viewer for `v1-roadmap.json` to review roadmap progress on an ongoing basis. Three `AskUserQuestion` calls clarified format/style/content-scope before building. Viewer generated as single self-contained HTML (kanban layout, inlined JSON, no server required). PMD reference entry #483 written so future sessions can find the file paths + open command without re-deriving. Main finding: the HTML viewer has its JSON inlined — it will silently drift from `v1-roadmap.json` as lanes ship unless explicitly re-synced. A re-sync workflow needs codifying.

---

## What surprised us

- **`memory_type: "reference"` is not a valid PMD enum.** The MCP schema accepts only `decision | bug | pattern | command | issue-note | deploy-note | qa-result | summary`. Used `pattern` instead. This is a recurring friction point: the MEMORY.md and lessons use "reference" as a conceptual category but the underlying MCP tool does not expose it. Future PMD writes for reference-type content should default to `pattern` without attempting `reference` first.

- **AskUserQuestion multi-select worked cleanly for content-scope selection.** Three questions asked up front produced enough signal to build the right output first-try — no back-and-forth corrections after generation. The clarify-before-build pattern paid off for a moderately complex UI deliverable.

- **The viewer JSON is inlined, not linked.** The HTML file reads `v1-roadmap.json` at build time (copy-paste into a JS const) not at runtime. This means the viewer will silently show stale data as the roadmap JSON is updated by `/auto-roadmap`. This was not discovered until after generation — it's a design gap that should be addressed at next re-sync.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add a "re-sync the roadmap viewer" instruction to PMD entry #483 and note that the JS `ROADMAP` const must be updated after every `/auto-roadmap` run | Future sessions know exactly what to do after a JSON update without re-deriving | minor (edit existing PMD entry) | 1× this session — new artifact |
| 2 | Consider switching the HTML viewer to fetch `v1-roadmap.json` at runtime via `fetch('./v1-roadmap.json')` so the viewer is always live | Eliminates inlined-data staleness; viewer auto-reflects JSON edits on refresh | minor–medium (rewrite the HTML's data loading block) | 1× this session — design gap spotted at generation |
| 3 | Add `"reference"` to the conceptual PMD type → `pattern` mapping to MEMORY.md or a lesson, so future advisor sessions don't attempt the invalid `reference` enum | Saves one round-trip MCP error per reference-type write | trivial (one-line note in MEMORY.md) | 1× this session, likely recurred silently in prior sessions |

## What to carry forward

- **Clarify-before-build via AskUserQuestion for UI/tool deliverables.** Three targeted questions (format / style / content scope) before any code generation produced a first-try acceptable result. Worth repeating for any non-trivial HTML/script deliverable.
- **PMD reference write immediately after creating a durable artifact.** Writing PMD #483 right after generating the viewer means the next session doesn't need to search for the file. Pattern: create artifact → write PMD entry → done.
- **`start "" "<path>"` opens files in default browser on Windows** without needing a dev server. Simple and reliable for local HTML viewers.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| AskUserQuestion (3 calls, pre-generation) | 10 | 0 | low | Clarified format/style/scope; prevented post-generation rework |
| HTML generation (single Write) | 20 | 0 | none | Viewer matched requirements first-try |
| PMD write (memory_write) | 5 | 2 | low | `reference` enum invalid → one retry with `pattern`; 2 min wasted |
| `start ""` browser open (Bash) | 2 | 0 | none | Clean one-liner, worked immediately |
| ToolSearch for memory_write schema | 1 | 0 | none | Routine schema load |

## Complexity scores (heavy tasks only)

No impl tasks ran this session. No complexity scores applicable.

## Decisions to revisit

- **Live-load vs inlined data in the viewer.** Inlined JSON is simple and works offline; `fetch()` approach requires the HTML to be served (even a `file://` URL may block `fetch()` on some browsers due to CORS). Worth testing the `fetch()` approach before committing to it — may need a tiny local HTTP server or a `file://` CORS workaround. Defer until next roadmap update triggers the need.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **`memory_type: "reference"` is invalid in PMD MCP — use `pattern`**: add a one-line note to MEMORY.md under the "References" section header. Trivial; prevents recurring MCP error on reference-type writes.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
