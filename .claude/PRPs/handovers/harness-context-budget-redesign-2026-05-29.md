# Handover — harness context-budget redesign (2026-05-29)

**For:** the next advisor session resuming this work.
**Branch:** `governance-v0` (canonical `brehon-fork` checkout — meta-edits only; do NOT touch phase branches).
**Read with zero conversation context — everything you need is below.**

---

## RESUME BLOCK

### The goal (user-set)

Cut the Claude Code **always-load Memory-files baseline from ~71k tokens → as close to ≤22k as the Pi dual-harness contract allows**. Total session baseline target ≤50k (fixed non-memory load ≈28k). The deep audit proved **≤22k is structurally impossible** (Pi-citation freeze map + irreducible cross-harness DQ-schema/gate-safety policy); the **defensible floor is ~50k**, reachable via a ~17–20k structural relocation. User accepted "Execute B1+B2+B3 + MEMORY rewrite (hit ~50k)".

### Authoritative documents (read these first)

1. `.claude/PRPs/reports/harness-redesign-session-profiles-2026-05-29.md` — **THE plan.** §2 = the exhaustive Pi-citation freeze map (which `##`/`###` headings are FROZEN vs relocatable). §3 = relocation plan with token deltas (B1/B2/B3). §5 = why ≤22k is impossible + the ~50k floor justification. **The freeze map in §2 is the gating constraint for every edit.**
2. `.claude/PRPs/reports/harness-audit-2026-05-29.md` — predecessor conservative-trim audit (applied `f9982bd6e`, ~1.8k won). Its "Execute-pass actuals" section has the **load-bearing token-math correction: live tokens ≈ chars × (1/2.4), NOT chars/4.** Use 2.4 chars→token for all markdown estimates.

### What's DONE (committed on governance-v0 this session)

| Commit | What |
|---|---|
| `bb0846683` | **Guard shipped** — `scripts/brehon/verify-rule-anchors.sh` (the safety net). Scans every cross-harness section-name citation of the 5 dual-harness rule files; verifies each cited heading resolves. Also restored the `### When to override` stub in advisor-orchestrator.md §5.1 (pre-existing dangle). |
| `7b7a2ab98` | **B1 COMPLETE** — relocated advisor-orchestrator.md §4 (cohort dispatch) + §5.3 (§G4 classifier tables) + §2.1 (brief-location) MECHANISM to `refs/auto-phase.md`. Kept all frozen heading stubs + pointers. advisor-orchestrator.md: 47,410 → 33,207 chars (−5,900 tok). |
| `0e3d52f90` | First handover (this file, now updated). |
| `206d1740d` | **B2 COMPLETE** — relocated decision-queue.md per-kind routing matrix (~38 lines) to `refs/dq-mechanics.md`. Kept all 5 Junior-pinned frozen anchors resident. decision-queue.md: 29,181 → 27,678 chars (−625 tok; smaller than the ~3,900 estimate — historical sections were already pointer-stubbed in a prior trim, so only the routing matrix remained relocatable). |

### IMPORTANT — Pi-removal recompute (this session, supersedes the report's §5 assumptions)

The user confirmed **Pi-Coding is dead** (no longer used) and asked whether removing it
would lower the floor. **Inline re-attribution proved it does NOT.** Two findings, both
load-bearing for the next session:

1. **Deleting `.pi/` cuts ~0 Claude-Code-session tokens directly** — `.pi/` is the OTHER
   harness's context; it never auto-loads into a Claude Code session. Verified footgun-free:
   NO `.pi/` path is wired into `.claude/settings.json` or `.mcp.json` (a few `.claude/`
   files mention `.pi/` in prose — harness-audit, session-retro, advisor-orchestrator — those
   are stale-able descriptive refs, not breakage).
2. **Removing Pi unfreezes ~0 load-bearing headings.** Every anchor Pi cited is INDEPENDENTLY
   cited by a LIVE non-Pi citer: the Junior subagent contracts (`.claude/agents/ci-watcher.md`
   pins `ci-watcher mutation pattern` + `kind:`; `agents/{bm-task,branch-manager,ci-watcher,
   impl-task,planning}.md` pin `Attribution integrity`; `agents/impl-task.md` pins `Forbidden
   execution windows` + `When to override`), live commands (`brehon-clarify/verify` pin
   `Stage-shape orchestration`, `Catch-fire procedures`), and crucially `.claude/agents/
   branch-manager.md:13` makes the SAME resident-by-inheritance assumption Pi did — so
   branch-manager.md stays always-load with or without Pi.

**Conclusion: the ~50-54k floor is JUNIOR-pinned, not Pi-pinned.** Pi removal is pure
housekeeping (dead 1.3M surface), NOT a budget lever. The 50k target is hit via B3 + MEMORY
exactly as planned. The user chose "B2+B3+MEMORY now; defer Pi decision" — so **the Pi-retire
call is still open; do not delete `.pi/` without re-confirming.**

### STATUS: COMPLETE (2026-05-29 session 2). Floor reached + harness-validated.

All planned levers applied + the two follow-on levers (B5a stale-trim, B5b
profile-scope) investigated and resolved.

**HARNESS-VALIDATED FINAL (via `/context`, fresh session, Opus 4.6):
Memory files = 45k tok (22.5% of 200K).** Under the 25%/50k threshold. Good.

**⚠️ TOKENIZER-MATH CORRECTION — the report's 2.4 chars→token ratio is WRONG.**
Measured directly against the harness's own per-file `/context` numbers, the
real ratio is **~3.3 chars/token** (advisor-orchestrator 33,207 ch → 10.1k tok
= 3.29; decision-queue 3.40; multi-lane 3.35; consistent across all files). So:
- Every "≈ Δ tok @2.4×" figure in this handover OVER-counts by ~1.4×. The
  CHAR deltas are accurate; divide chars by ~3.3 (not 2.4) for real tokens.
- The "71.5k → 62.2k" framing used the bad ratio on both endpoints. At the true
  ratio the baseline was ~52k and we are now at **45k** — a real **~−4k tok /
  ~−13k char** cut. The relative cut (~13%) is right; the absolute token
  numbers below are inflated — trust the 45k harness figure.
- Do NOT re-propagate 2.4 into any future harness-audit; use 3.3 (Opus-4.6
  tokenizer) or just read `/context` directly.

This is at the defensible floor; no further structural lever exists for this
dual-harness repo. Details below.

### ⚠️ DO NOT re-chase these (verified dead ends — proven, not assumed)

1. **Profile-scoping (B5b) is STRUCTURALLY IMPOSSIBLE.** The predecessor report's
   "~15-22k from making impl/BM sessions skip advisor-orchestrator.md etc." is a
   MIRAGE. Verified against source this session:
   - The ONLY rule-scoping primitive is `paths:`-on-Read (empirically confirmed,
     `.claude/lessons/reference_claude_code_rules_loading.md:13`). It fires on a
     file-Read. The three big files' load conditions are session-role (advisor)
     and worktree-count (multi-lane) — neither is a file-Read. No glob means
     "this is an advisor session."
   - NO role-based rule-loading config exists (`settings.json`/`.local.json` =
     permissions + hooks only). Subagents inherit the full corpus.
   - `branch-manager.md` is Pi-pinned: `.pi/skills/branch-manager/SKILL.md:12`
     tells Pi's BM agent the file is "already loaded… through project-rules
     inheritance" and not to re-read it. Scoping it out silently strips Pi's BM
     agent of all file-ownership/refusal discipline.
   - **Conclusion: the only structural lever is refs-relocation (B1/B2/B3), now
     exhausted.** Do not re-open profile-scoping.
2. **The freeze map in `harness-redesign-session-profiles-2026-05-29.md` §2 is
   STALE for multi-lane.** It said `§"Lane modes"` + `§"Brief location and
   trunk→phase sync"` were relocatable (0 citations). They are NOT — B1 added
   Claude-side citations to them (`refs/auto-phase.md:586,588`) THIS effort, so
   they became frozen mid-stream. **The `verify-rule-anchors.sh` guard is the
   ground truth, NOT the report.** Always run it before trusting any freeze call.

### What was DONE (all on governance-v0)

| Commit | Lever | Δ tok |
|---|---|---:|
| `7b7a2ab98` | B1 advisor-orchestrator → refs/auto-phase | −5,900 |
| `206d1740d` | B2 decision-queue routing matrix → refs/dq-mechanics | −625 |
| `d52fd5811` | B3 multi-lane rationale → refs + PMD-cross-lane compress | −823 |
| (MEMORY.md, not repo-tracked) | MEMORY.md prune: Historical ledger collapse + 6 verbose entries shortened | −1,475 |
| `22418bad6` | B5a decision-queue stale-trim (deprecated Shape-G kinds, two-entry-design, Phase-6-#37 rationale, refusal #7 compress) | −498 |

MEMORY.md: 26,318 → 22,781 bytes (under the 24.4 KB hard budget; was truncating).

### Running total (final)

| Step | ≈ Δ tok @2.4× | Status |
|---|---:|---|
| B1 advisor-orchestrator | −5,900 | done `7b7a2ab98` |
| B2 decision-queue routing | −625 | done `206d1740d` |
| B3 multi-lane | −823 | done `d52fd5811` |
| MEMORY.md prune | −1,475 | done (user-scope) |
| B5a decision-queue stale-trim | −498 | done `22418bad6` |
| **Cumulative** | **~−9,300** | **71.5k → ~62.2k (31% of 200K window)** |

**Why not ≤50k:** B2 was already-trimmed (−625 not −3,900); B3 hit the frozen
`Lane modes`/`Brief location` wall (−823 not −4,170); B5b (the big ~15-20k hope)
is impossible. The ~62k floor = the Pi-frozen DQ contract (~8k) + resident-by-
inheritance branch-manager (~5k) + gate/safety policy + ADR substrate. Going
lower means breaking Pi or dropping four-role safety discipline. **The durable
lever from here is growth-discipline** (new mechanism authored in refs/ from the
start — the `rule-narrative-bloat-reminder.sh` hook + advisor-orchestrator §3.6
already enforce it), NOT another relocation pass. v1 nears completion (5 lanes
done / 3 partial); the orchestration apparatus retires when v1 closes, which is
the only thing that meaningfully shrinks this corpus further.

**Honest note on the target:** even with all of B1+B2+B3+MEMORY, the floor lands ~55-58k, not a hard 50k — B2 under-delivered (already-trimmed) and the irreducible Junior-pinned contract (~13k of decision-queue + the resident-by-inheritance branch-manager ~5k + the gate/safety policy) is the wall. To get UNDER 55k you'd need to either (a) accept it, (b) attack the floor files / MEMORY.md harder, or (c) revisit whether branch-manager.md's resident-by-inheritance assumption can be converted to an explicit Read in `agents/branch-manager.md` (would free ~5k but needs testing the foreground BM still works). **Validate the actual number via `/context` in the new session before deciding if further cutting is worth it.**

### The METHOD (follow exactly — it's how B1 stayed safe)

1. For each chunk: read the section in the rule file, append it (verbatim, with a relocation provenance note) to the destination refs file, then shrink the rule section to **frozen-heading-stub + terse statement + pointer to the refs §**.
2. **Keep every FROZEN heading as a heading** (per §2 freeze map). Pi cites headings by prose name — moving/renaming one breaks Pi silently.
3. **Keep SAFETY policy resident, move only MECHANISM.** (B1 kept §3.2 gates, cycle-count meta-rule, catch-fire table, DQ triage tree resident; moved only the step-by-step tables.)
4. After each rule file's edits: run `bash scripts/brehon/verify-rule-anchors.sh --list`. **The baseline has exactly 3 known-benign dangles** (compound-name/phrase artifacts, NOT real breaks): `decision-queue.md §"Attribution integrity §Detection"`, `decision-queue.md §"Recipe 2 self-resolved"`, `multi-lane-worktree.md §"multiple active files"`. **Any NEW dangle = you broke a citation; fix before committing.**
5. Commit each B-step separately with measured char/token delta in the commit body.

### VALIDATION (optional — the chars→tok math below is already done)

The post-session number was computed directly from char counts at 2.4 chars→tok
(the verified live ratio): **~62.2k always-load** (rules 118,981 ch ≈ 49.6k +
CLAUDE.md 7,467 ch ≈ 3.1k + MEMORY.md 22,781 ch ≈ 9.5k). If you want the
harness's own figure, run `/context` in a FRESH session and read the **Memory
files** bucket (not total — `/context` inflates "Messages"). Expect ~62k, NOT
the report's stale ~50-54k target (that assumed B5b profile-scoping, which is
impossible — see "DO NOT re-chase" above).
- Per-file now: advisor-orchestrator ~13.8k (was 19.8k), decision-queue ~11k
  (was 12.3k), multi-lane ~6.5k (was 7.3k), MEMORY.md ~9.5k (was 11.4k).

### Tasks (final — all resolved)

- B1 advisor-orchestrator — **done** `7b7a2ab98`
- B2 decision-queue routing — **done** `206d1740d`
- B3 multi-lane — **done** `d52fd5811`
- MEMORY.md prune — **done** (user-scope, not repo-tracked)
- B5a decision-queue stale-trim — **done** `22418bad6`
- B5b profile-scope — **closed NOT-VIABLE** (structurally impossible; verified)
- Guard (`verify-rule-anchors.sh`) — **done** `bb0846683`

### Cross-session notes / hazards

- **Lane topology:** `git worktree list` shows `brehon-fork-redaction-r1` (phase-v1-redaction-r1) + `brehon-fork-rt-r4` (phase-v1-RT-r4) active. This work is governance-v0 meta-edits in canonical only — no phase-branch contact. No conflict, but other sessions may be driving those lanes.
- **Stray deletion (NOT mine, NOT addressed):** `.claude/PRPs/plans/v1-data-model-doc-and-04-retire.plan.md` is deleted in the working tree (was committed in `96b86a8de` this session, then removed). Left untouched — may be intentional cleanup from the data-model-doc consolidation. If accidental: `git checkout -- <path>`. Surfaced to user; unresolved.
- **Active Brehon work this session (do not disturb):** v1-quality-r2 Task 0 (#509) was running per the cold-resume runlog; v1-RT-r4 just cut. This redesign is orthogonal infra work.
- **The guard is now wireable as a PreToolUse hook** on rule edits (future) — currently standalone. Run it manually after each relocation.

### Honest framing to carry forward (from the audit)

The one-time structural win is **~17-20k tokens (10× the conservative trim's 1.8k)**, landing at **~50-54k**, NOT 22k. The durable lever is **growth-discipline** — new mechanism/incident-narrative authored in `refs/` from the start (the `rule-narrative-bloat-reminder.sh` PostToolUse hook + advisor-orchestrator §3.6 already enforce this). The `verify-rule-anchors.sh` guard now makes the Pi-freeze map enforced, not just documented.
