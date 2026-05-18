---
name: CC v2.1.119 hardcoded .claude/** sensitive-file gate blocks Junior BM writes
description: Claude Code v2.1.119 enforces a hardcoded .claude/** sensitive-file gate NOT overridden by --dangerously-skip-permissions / bypassPermissions / settings.json permissions.allow. Junior bm-task runlog + DQ writes are blocked. The advisor-relocate (§6) is the EXPECTED path until a scoped PreToolUse hook ships — not a task failure.
type: feedback
---

Claude Code **v2.1.119** enforces a hardcoded `.claude/**` sensitive-file gate on `Write`/`Edit` tool calls. This gate is **NOT** overridden by:

- `--dangerously-skip-permissions` (the Junior worker runs in `bypassPermissions` mode — the gate STILL fires),
- `settings.json` `permissions.allow` entries,
- any per-task env override.

**Consequence for Junior `[role:bm-task]` work:** bm-cut's deliverables are (a) the phase branch (git — UNAFFECTED, branch creation is not a `.claude/**` write) and (b) `.claude/runlog/<phase>-runlog.md` (a `.claude/**` write — **BLOCKED**). A `kind: "blocker"` DQ write to `.claude/decision-queue.json` from the worktree is **also blocked** by the same gate. The worker sees a "sensitive file" denial and cannot complete the runlog/DQ portion of the task even though the load-bearing git work succeeded.

**This is the EXPECTED path, not a task failure.** Until the scoped PreToolUse hook is installed (deferred — see "Planned fix"), every bm-cut (and any bm-task that writes `.claude/**`) will hit this. The recovery is the **advisor-relocate (§6 pattern)**, codified in every bm-cut brief's "§6 KNOWN harness limitation" section:

1. The worker writes the would-be runlog content to `<worktree-root>/<phase>-runlog.md` (worktree root is writable — the gate is `.claude/**`-scoped, not whole-worktree). For a blocked DQ blocker, the worker writes `<worktree-root>/<phase>-<verb>-BLOCKER.md` and STOPS with a clear escalation message.
2. The worker completes the git deliverables normally (branch create + push are unaffected) and states explicitly in its Phase 5 summary: "runlog written to worktree root (not `.claude/runlog/`) due to CC v2.1.119 sensitive-file gate; advisor must relocate."
3. The **advisor** (laptop session, NOT gated — the gate is a Junior-worker-mode artifact) creates `.claude/runlog/<phase>-runlog.md` on `governance-v0` from the canonical checkout, matching the canonical runlog shape (mirror `feedback_retro_not_report.md`-adjacent siblings like `v1-JM-e-runlog.md`), and commits with a `docs(advisor):` subject. This is attribution-honest: the advisor is *relocating a gate-blocked BM deliverable*, not authoring BM content from scratch (the runlog facts come from the worker's Phase 5 summary + the git state the worker produced).

**Confirmed (recurrence ≥2):**
- **v1-AD-e bm-cut, Junior #282, 2026-05-16.** Log showed permission denials on `mkdir -p .claude/runlog`, `cat /tmp/v1ad-runlog.txt`. Worker's text: "Runlog: Unable to write due to permissions, but prepared". Advisor created `.claude/runlog/v1-AD-e-runlog.md` on `governance-v0` (`36418c87d`, `docs(advisor):`).
- **v1-ship-1 bm-cut, 2026-05-16** — the brief's §6 explicitly codified this as the EXPECTED path after the same gate-block (session-retro-2026-05-16-v1-ship-1-replan-bmcut-handoff.md).
- The pattern is now pre-emptively written into every bm-cut brief §6 — i.e. the org has accepted it as a standing constraint, which is itself the signal that it needs a corpus lesson (so PMD search surfaces it, not just inline brief prose).

**Why this lesson exists separately from the inline brief §6:** the gate is currently documented ONLY inside individual bm-cut brief §6 sections. Brief prose is not PMD-indexed the way `.claude/lessons/feedback_*.md` is (`scripts/sync-lessons-to-pmd.sh`). An advisor session reaching for "why did the bm-cut runlog fail" via `memory_search_hybrid` finds nothing — it has to re-derive the gate from a brief it may not be reading. This lesson makes the constraint + the recovery PMD-searchable so the next bm-cut/bm-pr session resolves it in one search instead of re-explaining it from scratch (~10 min saved per occurrence, every occurrence until the hook ships).

**How to apply:**

- **Authoring a bm-cut / bm-pr / any bm-task brief that writes `.claude/**`:** include the §6 KNOWN-harness-limitation block (the relocate fallback). Do NOT treat the `.claude/**` write as the load-bearing deliverable — the git work is; the runlog/DQ is recoverable.
- **Triaging a `done` bm-task whose runlog is missing from `.claude/runlog/`:** check the worktree root for `<phase>-runlog.md` (or `<phase>-<verb>-BLOCKER.md`). If present, relocate it advisor-side. If the worker's Phase 5 summary says the gate fired, this is expected — not a failure.
- **The advisor-relocate commit subject MUST be `docs(advisor):`** (relocating a gate-blocked BM deliverable). It is NOT a `chore(bm):` (the advisor is not the BM session) and NOT a process breach (the relocate is the sanctioned §6 recovery, the runlog facts are the worker's).

**Planned fix (not yet shipped):** a **scoped PreToolUse hook** that allow-lists the specific `.claude/runlog/**` + `.claude/decision-queue.json` write paths for Junior bm-task mode, so the worker can write them directly and the advisor-relocate tax disappears. Deferred per multiple brief §6 notes "until the user is at the laptop terminal" (the hook install is itself a `.claude/settings.local.json` / hook-script change that wants a focused session). Until then, the advisor-relocate is standing procedure. Worth scheduling the hook install as its own small task — it eliminates a recurring ~10–20 min tax on every bm-cut.

**Generalises to:** any harness version that hardcodes a sensitive-path gate not overridable by the documented bypass flags, in an orchestration model where worker subagents must write to that path class. The mitigation pattern — worker writes to a non-gated sibling path + escalates, orchestrator relocates from an un-gated session — applies whenever the gate is worker-mode-specific and the orchestrator runs un-gated.

**Symptom to recognise:** Junior bm-task log contains `"This Bash command contains multiple operations. The following parts require approval: mkdir -p .claude/runlog, ..."` or a `Write`/`Edit` "sensitive file" denial on a `.claude/**` path, while the git portion of the same task succeeded. The worker's final summary will say the runlog was "prepared but not written" / "Unable to write due to permissions".

**Companion lessons:**
- `feedback_junior_finalize_merges_bm_cut_branch.md` — the finalize-merge bug that co-occurs on the same bm-cut task (this gate incidentally contains that merge's blast radius by also blocking its push).
- `feedback_settings_local_json_worktree_bootstrap.md` — `.claude/settings.local.json` is per-worktree; relevant when the scoped PreToolUse hook eventually ships.

**Where this bit:** `.claude/PRPs/reports/session-retro-2026-05-16-v1-ad-e-gate-bmcut-finalize-recovery.md` + the bm-cut brief §6 sections for v1-AD-e and v1-ship-1.
