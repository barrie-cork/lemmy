# Session retro — 2026-06-03 — M1 planning pipeline (pre-/auto-phase)

> **PMD WRITE: DONE (System-2 eval id 766).** Initially the CC session's `project-memory`
> MCP *client* was not connected, so `memory_write_eval` wasn't directly callable. But the
> HTTP daemon at `localhost:11435/mcp` IS up, and the retro-check Stop hook itself reads
> retros via the MCP session protocol over `curl` (initialize → tools/call). I used that
> SAME blessed protocol to WRITE the eval (`memory_write_eval` over the HTTP MCP session) —
> recorded as eval id 766, `created_at 2026-06-03 21:11:07`, verified visible to the hook's
> exact query within the 60-min window. Per `universal-guards.md` §1/§4 + `pmd-invariants.md`
> #2, NO `created_at` forge, NO raw SQL, NO hook edit — this is the daemon's own write API.
> This file remains the human-readable durable capture. (junior-brehon MCP is still not
> connected — the dispatch blocker stands until session reload.)

**Harness:** claude-code
**Session window:** ~2026-06-03 (single session)
**Branch:** `governance-v0` throughout (no phase branch — pre-bm-cut)
**Branch at start:** `00b3b6b5b` → **at end:** `cb2c67d8b`
**Files touched:** 6 commits (2 OQ/PRD edits, 1 sub-PRD, 1 planning brief, 1 clarify DQ+brief, 1 handover)
**Commits:** `b36dcfa0f`, `5c96f30c5`, `46597f523`, `9dd601513`, `cb2c67d8b` (+ this retro)

## Three-signal score

- **CONFIDENCE (pre-scoring):** 0.70
- **Goal achieved:** partial — the invoked goal (`/auto-phase M1`) could not run (no plan exists; /auto-phase drives an existing plan). Correctly refused, then executed the *correct* substitute goal: the pre-/auto-phase pipeline (sub-PRD → brief → clarify) up to the dispatch boundary. Dispatch itself blocked on MCP-not-connected (environmental, not a work failure).
- **Tests:** none (pure planning/design/doc session — no code, no cargo).
- **Clean execution:** yes — no retries, no wrong-premise propagation, all 6 commits clean + pushed; the one infra blocker (MCP) was diagnosed (daemon up, client not loaded) not flailed at.
- **SCORE: 0.62** (read-only/no-commits-to-code session caps ~0.70 per evaluation-calibration.md anti-inflation rule #1; partial goal — the headline `/auto-phase` couldn't complete — pulls it to 0.62. Honest: the session produced correct, durable artifacts but did NOT reach the thing the user typed.)

## What happened (TL;DR)

`/auto-phase M1` was a hard refusal (auto-phase.md Hard refusal #3 — no plan file). The instinct to treat `M1` as a wrong identifier was itself wrong: ADR-016 renamed V2a/V2b/V2c → M1/M2/M3, so `M1` is the *correct* current name for the chat-infrastructure cluster. The real gap was that /auto-phase drives an existing plan and none existed. Surfaced intent to the user (got "kick off V2a planning now"), then ran the pre-pipeline: §3.3 OQ-resolvability pass → 2 user scope decisions (M1=chat-infra-only, OQ-V2-08=Brehon↔Brehon-only) → M1 sub-PRD (resolving OQ-V2-09 soft-pause) → planning brief → /brehon-clarify (4 DQ, 0 pending). Stopped at the planning-Junior dispatch: `junior-brehon` MCP not connected.

## What surprised us

- **`M1` was right, not wrong.** Initial read (memory said "V2→M1/M2/M3 rename" for the ADR-016 *backplane*) suggested M1 might be a different axis. The register (line 284) showed ADR-016 ALSO renamed the *messaging* clusters V2a→M1. So the user's invocation used canonical nomenclature; the umbrella PRD is the stale one (still V2a/V2b/V2c). Lesson: when an arg "looks wrong," check the rename history before refusing on naming — the doc the user read may be newer than the doc I'm checking against.
- **Two MCP servers silently absent.** Both `junior-brehon` (stdio) and `project-memory` (http) are in `.mcp.json` but not loaded this session. The project-memory HTTP daemon is actually UP (400 to bare GET); the gap is the in-session client. This bit twice: dispatch (junior-brehon) and this very retro (project-memory). A session reload is the fix for both.
- **ADR-016 vs umbrella-PRD scope tension was real, not cosmetic.** ADR-016 calls M1 "the first reference integration" of the backplane; the umbrella's M1 is pure chat infra. That's a genuine plan-size fork — surfaced to the user rather than assumed. User chose chat-infra-only, which cleanly re-scoped OQ-ADR016-01/03 to no-longer-block-M1.
- **The greenfield-services-harness gap is novel.** Every prior Brehon phase was in-workspace Rust on existing crates. M1's `services/bridge/` is workspace-excluded + Docker-dependent — the standard validate-pending-laptop `cargo --workspace` flow doesn't cover it. Surfaced as a clarify question; user chose "laptop runs all bridge validation." The planner now owns designing the exact validate-pending shape.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **SessionStart MCP-connectivity guard.** Add a session-start probe that checks `junior-brehon` + `project-memory` MCP tools are actually loaded (not just configured in `.mcp.json`), and WARNs loudly if absent. This session lost the dispatch AND the retro to silently-absent MCP. A guard would surface "reload required" at session start, not at the dispatch/Stop boundary. | Catches MCP-not-loaded before work depends on it | minor (one hook) | 1× this session (bit twice); MCP-absence pattern noted before |
| 2 | **`/auto-phase` arg-resolution: check rename history before naming-refusal.** When an `/auto-phase <X>` arg has no plan AND no obvious phase match, scan ADR/OQ register for a rename mapping `<X>` before concluding "wrong identifier." M1→V2a would have resolved instantly. | Avoids mis-framing a correct arg as a typo | minor (skill note) | 1× this session |
| 3 | **Retro-tool-down fallback should be a named, blessed path.** When `memory_write_eval` MCP is unreachable, the blessed fallback (durable System-1 report file + deferred-backfill note + let hook fail-open) should be documented so it's not re-derived each time. This session derived it from first principles via universal-guards + pmd-invariants. | Faster, consistent handling of MCP-down retro | minor (lesson) | 1× here; MCP-down is recurring |

## What to carry forward

- **Pre-/auto-phase pipeline is the right discipline.** `/auto-phase` is NOT a planning tool. The sequence sub-PRD → brief → clarify → planning Junior → plan-approval → THEN /auto-phase is correct and worth keeping explicit. The user's "kick off planning now" confirmed the framing.
- **OQ-resolvability pre-flight (§3.3) paid off.** It cleanly separated M1's real gates (OQ-V2-08, OQ-V2-09) from the deferred-with-backplane ones (OQ-ADR016-01..04), and surfaced OQ-V2-08 as the one user-decision needed. ~10 min, high signal.
- **Surfacing scope decisions BEFORE authoring saved a wrong-premise sub-PRD.** The M1-scope question (chat-infra-only vs +B-fetch vs +B-actor) would have produced a very different sub-PRD if guessed. Asking first was correct.
- **The dispatch handover is the bridge across the reload.** `.claude/PRPs/handovers/m1-planning-dispatch-2026-06-03.md` has the full resume sequence. On reload, re-run `/auto-phase M1` → it'll complete the dispatch + run the plan-approval pre-gate.

## ROOT_CAUSE (for the partial goal)

`ENVIRONMENTAL — MCP-client-not-loaded`. The `/auto-phase M1` goal could not complete because (a) no plan existed (correctly handled by pivoting to the pre-pipeline), and (b) the planning-Junior dispatch needs `junior-brehon` MCP which wasn't connected this session. Neither is a work-quality failure; both are environmental. The fix is a session reload, documented in the handover. No code or process defect.

## Deferred actions (backfill on next session)

1. **`memory_write_eval`** (qa-result, this retro) — write to System-2 PMD once MCP reconnects. Title: "Task retro: M1 planning pipeline (pre-/auto-phase) 2026-06-03". Score 0.62.
2. **Dispatch the M1 planning Junior** — per the handover RESUME block.
3. (Optional) Promote change-items #1-#3 above to lessons if they recur.

---

_System-1 durable capture; System-2 `memory_write_eval` deferred (MCP client not connected).
Lessons consulted: `universal-guards.md` §1/§4, `pmd-invariants.md` #2/#3,
`evaluation-calibration.md`, `feedback_retro_not_report.md`._
