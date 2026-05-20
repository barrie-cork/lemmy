# v1-rls-r1 planning brief — RLS hardening Wave 1

**Written**: 2026-05-20 by advisor session (laptop, lane-dedicated worktree `C:/Users/barri/Developer/brehon-fork-rls-r1`, branch `phase-v1-rls-r1` @ `6e16ee94f`).
**Subagent target**: `planning` (Opus 4.7 — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/v1-rls-r1-planning-1` from `phase-v1-rls-r1` committed HEAD `6e16ee94f`. Plan file commits + pushes to `phase-v1-rls-r1` at finalize.
**Authority anchor**: this brief IS the canonical planning input for `v1-rls-r1`. It absorbs (a) the RLS-PMD review at `docs/research/brehon-rls-pmd-review.md` top-5 hardenings, (b) user decisions 2026-05-20 captured in §0.1 (BINDING and pre-empt any open-ended alternatives the review enumerates), (c) the multi-lane-worktree + retro-not-report discipline + canonical-schema-first conventions established post-fed-in-b.
**Sub-phase target**: `v1-rls-r1` — first Wave of Recursive-Learning-System (RLS) hardening. Ships **five** harness deliverables that together close the edges between the existing five-tier retro architecture and durable enforcement (item-codes match `docs/research/brehon-rls-pmd-review.md` §6): **(4.2-spec)** MCP write-time embedding spec (brehon-fork-side spec only — actual MCP server patch ships in a separate `MCPs/project-memory-mcp/` PR per PRECON-2), **(4.1)** SessionStart canonical-PMD guard (script + lane-bootstrap wiring), **(4.8)** PMD-invariants rule promotion (one tracked rule file consolidating five PMD invariants), **(4.6)** retro-harvest into weekly-review as mandatory Step 3 (skill edits), **(4.7)** governance-log every `retro-check.sh` fail-open (one shell function + governance-log-entry kind registration). Read-only audit infrastructure plus minimal targeted hook/skill writes; no `crates/**` impl writes; no migration; no Shape-G workflows; ships under `validate-pending-laptop` DoD.

---

## 0. Why this sub-phase exists (the design problem — read first)

The brehon-fork RLS (Recursive Learning System) is the trust-building substrate for the autonomy journey. Per `docs/research/brehon-rls-pmd-review.md` §1, the architecture is five layered retro tiers (per-task → session → sub-phase → weekly → harvest) on a two-system PMD substrate (auto-loaded markdown + queryable SQLite-vec DB), with a promotion ladder (observation → lesson → canonical pattern). The review's verdict (§Appendix): **the architecture is sound; the gaps are all at the edges between tiers, between systems, between spec and enforcement.**

Concrete evidence the edges are open (each item also evidenced in the PMD as a lesson or session retro):

- **2026-05-18 v1-ship-1 incident** (`.claude/PRPs/reports/session-retro-2026-05-18-pmd-stranding-remediation.md` + `feedback_pmd_cross_lane_canonical_db.md`): ~27+ false `retro-check.sh` Stop-hook blocks; **21 retros stranded** in a lane-local PMD; canonical-path invariant violated for an entire phase before anyone noticed. The documentary guard (`_comment_pmd_cross_lane` marker in `.mcp.json.example`) did not prevent it because nothing re-reads it. The `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` lesson records the concrete mechanism spec'd but **NOT shipped** (status: PENDING). v1-rls-r1 ships it (item 4.1).
- **2026-05-16 MCP write-time embedding gap** (`feedback_pmd_backfill_after_write.md`): source inspection of `dist/index.js` confirmed `memory_write_eval` writes the FTS5 row but **does not embed**. Every `post-task-retro` write is FTS5-only until the next `backfill.js` run. Worst-case Junior window: ~6.5 days. `memory_search_hybrid` silently degrades; the +62.7% Recall@10 hybrid advantage is absent during that window. v1-rls-r1 ships the **spec** for the patch (item 4.2-spec); the actual `dist/index.js` patch is deferred to a separate MCPs PR per PRECON-2.
- **2026-05-16 retro-harvest aggregation gap** (per RLS-PMD review §4.6): scan of `.claude/PRPs/reports/*.md` found ~48 unchecked proposals across 52 retro files. Among them, a `cycle_count ≥ 3` catch-fire proposal had been **7 days unread before it would have prevented a ~123-min loss** on v1-SL-c-2 cycle-3 (per `feedback_plan_stub_uniformity_with_canonical_sibling.md` § "cycles cost ~123 min"). The harvest tier (`retro-harvest` skill) is read-only + ad-hoc; nothing makes it run. v1-rls-r1 folds it into weekly-review (item 4.6) so cadence is guaranteed.
- **Multi-lane invariant scattering** (per RLS-PMD review §4.8): five PMD-meta lessons — canonical path, two-systems, backfill, LESSON-trailer, SessionStart guard — are scattered as recurrence-based lesson files. They are *invariants*, not heuristics. Scattering them mixes two categories of memory; new sessions cannot tell which to treat as principles vs absolutes. v1-rls-r1 promotes them to `.claude/rules/pmd-invariants.md` (item 4.8).
- **`retro-check.sh` fail-open bypass surface** (per RLS-PMD review §4.7): the Stop hook fails open after 3 attempts (loops are real). For autonomy this is a known hole — a determined agent gets a free third pass with no retro written and no trail. The behaviour stays (loops ARE real); v1-rls-r1 adds the trail (item 4.7).

These five gaps are not new code paths — they are **enforcement of existing invariants**. The RLS architecture is already correct; this sub-phase makes the boundaries of "spec'd but not enforced" disappear for the top-5-leverage items.

---

## 0.1 Pre-resolved facts (READ BEFORE §1 — these are BINDING; do NOT re-derive or file blockers for them)

The advisor resolved these in `AskUserQuestion` exchanges on 2026-05-20. They pre-empt round-trips the planner would otherwise have to make.

### 0.1.1 — PRECON-1 — Scope: TOP 5 hardenings, full slate

**User decision 2026-05-20 (Q1 — Scope):** "Top 5 hardenings — full slate (Recommended)". v1-rls-r1 ships all five top-5 items from the RLS-PMD review §6, fused into one plan via Track A / B / C structural split below. Items 4.3, 4.4, 4.5, 4.9, 4.10 from the review's §4 are deferred to a follow-up `v1-rls-r2`.

**Binding consequence for the plan:** §13 has THREE tracks fused into one plan:

- **Track A — DOC + SPEC artifacts** (no behaviour change, no compile dependency): items 4.2-spec (the MCP write-time embedding contract documented in `.claude/PRPs/specs/mcp-write-time-embedding.md`) + 4.8 (the `.claude/rules/pmd-invariants.md` rule file consolidating five PMD invariants). These ship first; they unblock the other tracks by giving them a single citation target.
- **Track B — SessionStart hook + skill edits** (behaviour-changing, brehon-fork-local): items 4.1 (the `.claude/hooks/pmd-canonical-guard.sh` script + the lane-bootstrap-checklist update so each lane wires the SessionStart entry on creation) + 4.6 (edits to `.claude/skills/weekly-review/SKILL.md` adding the retro-harvest Step 3 + an output directory `.claude/harvest/`).
- **Track C — `retro-check.sh` governance-log instrumentation** (behaviour-changing, narrow scope): item 4.7 (a shell function appended to `retro-check.sh` that emits a `governance-log-entry` of kind `retro_bypass` on every fail-open, plus the registration of that new entry kind in the governance-log schema doc + the lesson cross-link).

The three tracks are independent and can ship as `[P]`-able task cohorts where FILES YAML disjointness holds. They are mutually reinforcing: Track A gives Tracks B+C the citation target for "what this enforces"; Track B closes the canonical-path and harvest gaps; Track C closes the bypass-trail gap. No cross-track compile dependency (Track A is pure markdown; Track B touches `.claude/hooks/` + `.claude/skills/`; Track C touches one hook script + one rule doc + one lesson).

### 0.1.2 — PRECON-2 — MCP server patch is OUT of scope (spec-only here)

**User decision 2026-05-20 (Q2 — MCP patches):** "Brief documents the MCP patch but doesn't ship it (Recommended)". The actual `dist/index.js` patch for write-time embedding lives in a separate repo (`C:/Users/barri/Developer/MCPs/project-memory-mcp/`) and would require cross-repo Junior orchestration plus an MCP-server release cadence that v1-rls-r1 cannot absorb without scope dilution.

**Binding consequences:**

- v1-rls-r1 ships **item 4.2-spec** — a `.claude/PRPs/specs/mcp-write-time-embedding.md` document that names the patch's contract: (a) the embedding call insertion point in `memory_write_eval` (post-FTS5 insert, pre-commit per the RLS-PMD review §4.2 + `feedback_pmd_backfill_after_write.md`), (b) the graceful-fallback shape (write the row, log the miss, let weekly-review's `1b` safety net catch it), (c) the verification recipe (run a 3-write probe; verify `SELECT COUNT(*) FROM memory_vectors` equals row count immediately after write), (d) the downstream-collapse claim (post-patch we delete session-retro Step 5.5's inline-backfill instruction and weekly-review Step 1b's safety-net branch).
- §12 "NOT building" enumerates "the actual `MCPs/project-memory-mcp/dist/index.js` patch — deferred to a separate `MCPs/project-memory-mcp` PR after v1-rls-r1 ships the spec; the v1-rls-r2 follow-up may carry the PR or it may ship as a meta-tooling commit outside the Brehon sub-phase cadence".
- Track A includes the spec doc but **does NOT** include the patch implementation. Track A's spec doc cites this brief as the authority for its scope.

### 0.1.3 — PRECON-3 — Rust best-practices research is excluded

**User decision 2026-05-20 (Q3 — Rust research):** "Excluded from v1-rls-r1; rest already absorbed by conformance-audit (Recommended)". The Claude-Code+Rust best-practices research at `docs/research/brehon-claude-code-rust-best-practices.md` top-5 items have already been absorbed by the parallel `brehon-conformance-audit` plan (rust-analyzer-MCP install + Clippy `disallowed_methods` for federation modules, per that brief's PRECON-3 + PRECON-4). The remaining items (`ENABLE_PROMPT_CACHING_1H=1` env-var rollout, cargo-nextest, CodeRabbit CLI) are tooling-adoption work that belongs in separate lanes.

**Binding consequence:** §12 "NOT building" enumerates these explicitly:

- No `ENABLE_PROMPT_CACHING_1H=1` worker-env rollout (Trigger: separate token-economy sub-phase).
- No cargo-nextest adoption (Trigger: follow-up plan after first conformance-audit calibration cycle).
- No CodeRabbit CLI generate→review→fix loop (Trigger: same).
- No further rust-analyzer-MCP scope beyond what conformance-audit already installs.

The v1-rls-r1 plan stays **RLS-PMD-review-only** — the lane name disambiguates from tooling-adoption lanes.

### 0.1.4 — PRECON-4 — Review point: user reviews AFTER clarify-gate, BEFORE planning-Junior dispatch

**User decision 2026-05-20 (Q4 — Review point):** "User reviews after clarify-gate, before planning queue (Recommended)". Standard Brehon four-role discipline preserved.

**Binding consequence:** the advisor's stage-shape for this brief is:

1. Author this brief (this file).
2. Commit + push to `phase-v1-rls-r1`.
3. Run `/brehon-clarify .claude/PRPs/briefs/v1-rls-r1-planning-1.md`.
4. Resolve every `kind: "clarify"` DQ raised (advisor-self-answer with citations OR user-relay per `feedback_clarify_before_plan.md`).
5. Surface the clarified brief to the user for approval (User Gate 1-pre — brief-level approval, separate from User Gate 1 plan-level approval that fires after the planning Junior task completes).
6. On user approval, queue the planning Junior task with the matching dispatch line in §1.
7. Planning Junior writes the plan; advisor runs §3.4 DoD smoke + §3.5 watchpoint-specificity + §3.7 dogfood gates; surfaces plan to user for User Gate 1 (plan approval).

This brief at `.claude/PRPs/briefs/v1-rls-r1-planning-1.md` IS the COMPLETE planning input for the Junior planning task. The Junior planning subagent (Opus 4.7) authors `.claude/PRPs/plans/v1-rls-r1.plan.md` per the §2 contract below.

### 0.1.5 — PRECON-5 — Five top-5 items are FIXED in v1-rls-r1

The five items (4.2-spec, 4.1, 4.8, 4.6, 4.7) from the RLS-PMD review §6 are the v1-rls-r1 schema contract. The planner does NOT add a sixth item (4.3, 4.4, 4.5, 4.9, 4.10 are explicitly deferred to v1-rls-r2). The planner does NOT drop a fifth (each is load-bearing per the review's leverage calculation). The five items, sourced VERBATIM from the review's §6 (the plan's §10 lifts this table):

| # | Item | What it ships | Source lesson(s) / PMD memory |
|---|---|---|---|
| **4.2-spec** | MCP write-time embedding spec | `.claude/PRPs/specs/mcp-write-time-embedding.md` — patch contract + verification recipe + collapse claim; NOT the patch itself | `feedback_pmd_backfill_after_write.md`; `project_pmd_no_embeddings.md` (PMD); RLS-PMD review §4.2 |
| **4.1** | SessionStart canonical-PMD guard | `.claude/hooks/pmd-canonical-guard.sh` (new tracked script) + lane-bootstrap-checklist update (per-worktree `settings.local.json` wiring) | `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` (the existing spec); `feedback_pmd_cross_lane_canonical_db.md`; v1-ship-1 retro |
| **4.8** | PMD-invariants rule promotion | `.claude/rules/pmd-invariants.md` — one tracked rule file consolidating canonical-path / two-systems / backfill / LESSON-trailer / SessionStart-guard as five numbered non-negotiable statements | `feedback_pmd_cross_lane_canonical_db.md`; `feedback_pmd_two_memory_systems_distinction.md`; `feedback_pmd_backfill_after_write.md`; the five PMD-meta lessons cited in RLS-PMD review §4.8 |
| **4.6** | retro-harvest into weekly-review | Edit `.claude/skills/weekly-review/SKILL.md` adding Step 3 ("retro-harvest sweep") + create output directory pattern `.claude/harvest/<iso-week>.md` + update `.gitignore` if needed | RLS-PMD review §4.6; `.claude/skills/retro-harvest/SKILL.md` (existing read-only skill is the consumed building block) |
| **4.7** | governance-log fail-open instrumentation | Append a shell function to `.claude/hooks/retro-check.sh` that emits a `governance-log-entry` of kind `retro_bypass` (with session id, attempt count, last assistant message hash) on every fail-open path + register the new kind in the governance-log schema doc + new lesson cross-linking the instrumentation | RLS-PMD review §4.7; `.claude/hooks/retro-check.sh` (the existing hook is the consumed surface) |

### 0.1.6 — PRECON-6 — Spec-not-code discipline for items 4.2-spec and 4.8

**The advisor's binding decision (2026-05-20, surfaced to user via brief-level approval):** items 4.2-spec and 4.8 are **specifications + rule files**, not active code paths. They take effect by being *cited* (Track A) — Track B+C wire to them, future briefs cite them, the `pmd-canonical-guard.sh` script reads `pmd-invariants.md` rule #1 as its authoritative target.

**Binding consequence:** Track A tasks ship as documentation-only commits — NO compile target, NO test, NO `cargo` invocation. The `validate-pending-laptop` DoD applies to the WHOLE sub-phase (per PRECON-7 below), but Track A's individual tasks do not produce cargo-runnable artifacts. The plan §15 sequence runs ONCE at the end, against the combined diff.

### 0.1.7 — PRECON-7 — `validate-pending-laptop` DoD shape, Shape-G suspended

Shape G is SUSPENDED until 2026-06-01 (per PMD `project_shape_g_suspended_2026_05_16`; DQ #229 pending re-enable). v1-rls-r1 ships under the `validate-pending-laptop` DoD pattern (per `.claude/rules/advisor-orchestrator.md` §5.2). The §15 cargo commands mirror `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` §15 (the laptop-shape exemplar).

**Binding consequence:** Track A tasks are documentation-only; Track B + C tasks add shell scripts + skill markdown + one hook function — none of these are Rust code. The §15 cargo invocations exist to confirm v1-rls-r1's NON-Rust deliverables did not accidentally break the workspace (e.g. a `.claude/settings.local.json` change should never break `cargo check`, but the gate catches accidents). The minimum §15 set:

- `bash scripts/brehon/cargo-check.sh --workspace --features full` — confirms no incidental breakage.
- `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings` — confirms no incidental clippy regression.
- `cargo test --test e2e --no-run -p lemmy_server` — sanity: phase-branch test-link still works.
- `bash .claude/hooks/pmd-canonical-guard.sh` (a probe-run from the lane-dedicated worktree) — confirms the new hook script exits 0 when `.mcp.json` is canonical AND emits a WARN+exit-0 when the script is pointed at a deliberately-wrong sentinel path. Both flavours captured to log.

Phase 2 e2e is NOT applicable (v1-rls-r1 ships no Rust changes); the planner explicitly states this in §15 and the §3.1 stage-shape skips the Phase 2 user gate.

### 0.1.8 — PRECON-8 — Dogfood: the SessionStart hook is dogfooded against THIS lane

**The advisor's binding decision (2026-05-20):** the §13 dogfood task for Track B Task 4.1 runs `pmd-canonical-guard.sh` against the *current lane* (`C:/Users/barri/Developer/brehon-fork-rls-r1/.mcp.json`) AND against a deliberate-mispoint sentinel created in `/tmp` (a hand-edited `.mcp.json` with a wrong `PROJECT_MEMORY_DB`). Confirms (a) zero false-positive on the legitimate canonical path, (b) loud WARN on the deliberate-mispoint. The dogfood report at `.claude/PRPs/reports/pmd-canonical-guard-dogfood-<YYYY-MM-DD>.md` documents both runs.

For item 4.6 (retro-harvest into weekly-review): the dogfood reads the last 30 days of `.claude/PRPs/reports/*.md`, runs the new Step 3 sweep, confirms it surfaces at least the cycle-count-≥3 proposal from `feedback_plan_stub_uniformity_with_canonical_sibling.md` (or the equivalent harvest target named in the retro corpus). The dogfood report names the proposals surfaced and the proposals it correctly skipped (already-promoted).

For item 4.7 (governance-log fail-open): the dogfood triggers a synthetic fail-open path by setting an artificial 3-attempt count via env var and confirms a `retro_bypass` entry lands in the governance-log JSONL (or equivalent — the planner names the exact governance-log file path per §4.6 below).

### 0.1.9 — PRECON-9 — Lane-bootstrap-checklist update is in scope but mechanism is limited

The `feedback_phase_lane_worktree_bootstrap_checklist.md` lesson + `feedback_settings_local_json_worktree_bootstrap.md` lesson establish that `.claude/settings.local.json` is per-worktree and gitignored. Wiring the SessionStart entry MUST therefore be a step in the bootstrap checklist, not a tracked-file edit. The planner's Track B Task for item 4.1 SHIPS:

- The tracked `pmd-canonical-guard.sh` script.
- An update to the checklist lesson at `feedback_phase_lane_worktree_bootstrap_checklist.md` adding a new step "(N+1) wire `pmd-canonical-guard.sh` SessionStart entry in `.claude/settings.local.json`" with the exact snippet to paste.
- A one-shot wiring of the entry in THIS lane's `settings.local.json` (the canonical lane the brief was authored in, which IS a multi-lane lane and benefits immediately).

The planner does NOT auto-wire the entry in OTHER active lanes (canonical `brehon-fork`, `brehon-fork-conformance-audit`, `brehon-fork-tooling`); §10 "Out of scope" enumerates this, with the note "lanes wire on their next bootstrap cycle per the updated checklist; for currently-active lanes the wiring is a follow-up commit the user runs at convenience". Per `feedback_principles_not_rules.md` — the script is the shared knowledge, the wiring is per-lane drift the bootstrap checklist catches.

---

## 1. Role + dispatch line

`[role:planning] v1-rls-r1 plan — RLS Wave 1 hardening: top-5 RLS-PMD review items (4.2-spec + 4.1 + 4.8 + 4.6 + 4.7) fused into one plan; Track A/B/C structural split; validate-pending-laptop DoD`

The actual create-task description (single line, <100 chars, per `.claude/rules/advisor-orchestrator.md` §2.1 Junior task description template):

```
[role:planning] v1-rls-r1 plan — see .claude/PRPs/briefs/v1-rls-r1-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at **`.claude/PRPs/plans/v1-rls-r1.plan.md`** following `.claude/PRPs/templates/plan.template.md`'s 20-section schema literally.

### 2.1 What this sub-phase ships (the deliverable surface)

The plan's §13 task list MUST cover exactly the following three tracks fused into one plan, and NOTHING beyond it (cross-check every task against §12 "NOT building" — anything not listed below is OUT):

**Track A — DOC + SPEC artifacts (no behaviour change):**

1. **`.claude/PRPs/specs/mcp-write-time-embedding.md`** — the item 4.2-spec deliverable. Per **DQ #298** (advisor-resolved), `.claude/PRPs/specs/` does NOT exist at HEAD `6e16ee94f`; Track A Task 1 ESTABLISHES the directory AND writes a 5-line `.claude/PRPs/specs/README.md` documenting its purpose: "Tracked specifications for contracts that live outside brehon-fork (e.g. MCP server patches, external-repo coordination docs). Each spec names the canonical implementation target, the contract verification recipe, and the downstream-collapse claim. Authored at brehon-fork-spec time; implemented in the target repo's own PR cadence." Then writes the spec doc itself with sections: (a) the canonical patch insertion point in `memory_write_eval` per RLS-PMD review §4.2 (post-FTS5 insert, pre-commit) with a code snippet showing the expected shape; (b) the graceful-fallback contract (write the row, log the miss with the exact log line format, let weekly-review §1b safety-net catch it on the next Sunday cycle); (c) the verification recipe (3-write probe + immediate `SELECT COUNT(*) FROM memory_vectors` should equal row count); (d) the downstream-collapse claim (post-patch, delete session-retro Step 5.5 inline-backfill instruction; weekly-review Step 1b becomes aspirational; the Junior 7-day window vanishes). The doc explicitly notes "actual `dist/index.js` patch ships in a separate `MCPs/project-memory-mcp/` PR; this doc is the contract that PR implements".

2. **`.claude/rules/pmd-invariants.md`** — the item 4.8 deliverable. Five numbered non-negotiable statements consolidating the PMD-meta invariants per RLS-PMD review §4.8. The five (the plan §10 lifts VERBATIM from §0.1.5 table above):
    1. **Canonical PMD path (absolute, cross-lane).** Every worktree's `.mcp.json` `PROJECT_MEMORY_DB` MUST be `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` — never relative, never per-lane. Detection: `bash .claude/hooks/pmd-canonical-guard.sh` (the v1-rls-r1 ships).
    2. **Two systems, one source of truth.** System 1 (auto-loaded markdown under `~/.claude/projects/.../memory/`) and System 2 (queryable SQLite-vec DB at the canonical path) are NOT interchangeable; never write retro content directly to System 1; never assume System 2 is loaded at SessionStart.
    3. **No write-time embedding (yet — see item 4.2-spec).** Every `memory_write_eval` writes the FTS5 row but NOT the vector; `backfill.js` must run between write and the next hybrid search OR the search degrades silently to FTS5-only. Mitigations: session-retro inline backfill (laptop-only); weekly-review §1b safety-net (Junior + safety net for missed). The `mcp-write-time-embedding.md` spec ships in v1-rls-r1; the patch ships separately.
    4. **LESSON-trailer discipline.** Every learning observation either fires a `LESSON:` trailer (in commit body) OR a `kind: "log"` DQ entry (mid-task push, resolved-immediately). Never both, never neither for a durable observation.
    5. **SessionStart canonical-PMD guard (post-v1-rls-r1).** The tracked `pmd-canonical-guard.sh` hook runs at every session start; lane drift surfaces as a loud WARN within seconds of session-start, not after a phase of stranding. Per `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`.

   Each invariant carries: a one-paragraph "Why this is non-negotiable" with the originating incident citation; a "How to apply" pointing at the script/hook/skill that enforces it (or the lesson that documents the manual mitigation if no enforcement exists yet); a `See also` block linking to the originating PMD-meta lessons. Cross-link these in MEMORY.md (in `~/.claude/projects/.../memory/MEMORY.md`) under a new "PMD invariants" promoted-pattern entry.

**Track B — SessionStart hook + skill edits (behaviour-changing, brehon-fork-local):**

3. **`.claude/hooks/pmd-canonical-guard.sh`** — the item 4.1 script per `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` §"Mechanism" (the lesson IS the spec — the planner lifts the Python one-liner + canonical-path resolution + WARN-not-FAIL exit-0 contract verbatim). Frontmatter: shebang `#!/usr/bin/env bash`, `set -euo pipefail`. Outputs a stderr WARN banner naming both the running lane's value AND the canonical target on mismatch; exits 0 always (per the lesson's WARN-vs-FAIL reasoning). Handles `.mcp.json` absent → exit 0 silently (absence resolves canonical by MCP default). The planner refines the exact one-liner against the running `.mcp.json` shape (verifies the key path `mcpServers."project-memory".env.PROJECT_MEMORY_DB` is still the canonical location).

4. **`.claude/skills/weekly-review/SKILL.md` Step 2c addition** — the item 4.6 deliverable. Per **DQ #296** (advisor-resolved), the new step is inserted as **Step 2c. Retro-harvest sweep** between current Step 2b (Lesson clustering) and Step 3 (Aggregate eval metrics) — adjacent to 2b because both are "promotion-candidate surfacing" operations against the lesson/retro corpus (NOT eval-metric aggregation); this placement preserves the existing Step 3-6 numbering verbatim (zero cascade). The Step 2c body: (a) `Glob .claude/PRPs/reports/*.md` for last 7 days; (b) for each retro, read the §"What to change" + §"Decisions to revisit" sections; (c) extract proposals NOT yet promoted to `.claude/lessons/` or `CLAUDE.md`; (d) write a single weekly artifact at `.claude/harvest/<iso-week>.md` with the proposals enumerated, each with a `(retro-source: <path>, proposal-text: <verbatim quote>, ground-truth-evidence: <if any>)` triple. The skill body explicitly notes this is **surfacing** not auto-promoting — manual review thereafter per the RLS-PMD review §4.6 contract.

5. **`.gitignore` entry for `.claude/harvest/`** — IF the planner decides the per-week harvest files belong gitignored (per the audit-metrics precedent — runtime journal data; summary lands in retro). Or the planner may decide tracked-with-prune-after-N-weeks per the lesson-promotion ladder discipline; PRECON-9 lean is gitignored per the audit-metrics analogue. The planner picks at plan time and files a `kind: "log"` DQ recording the choice.

6. **Lane-bootstrap-checklist update** — edit `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` adding the new step: "(N+1) wire `pmd-canonical-guard.sh` SessionStart entry in `.claude/settings.local.json`" with the exact JSON snippet. The planner reads the current checklist to choose the correct numbering (N+1 vs renumber). This task ALSO one-shot-wires the entry in THIS lane's `settings.local.json` (PRECON-9 — current lane benefits immediately; other lanes follow on their next bootstrap).

**Track C — `retro-check.sh` governance-log instrumentation (behaviour-changing, narrow scope):**

7. **`.claude/hooks/retro-check.sh` fail-open instrumentation** — the item 4.7 deliverable. The planner reads the existing `retro-check.sh` (it lives at `.claude/hooks/retro-check.sh`; per the RLS-PMD review §3 "branch-scoped enforcement" + verified at HEAD `6e16ee94f` lines 141-144 the fail-open path is the 3-attempts-then-`rm -f`-then-`exit 0` branch) and APPENDS a shell function `emit_retro_bypass_log` invoked from that fail-open branch. The function writes a JSONL record to a dedicated sidecar (per the brief's PRECON-9 lean; observation-capture.sh is structurally unsuitable because it writes per-PPID transient cache files at `~/.cache/tw-observations/<PPID>.jsonl` — gitignored shadow-mode runtime, NOT a durable trail; see `.claude/hooks/observation-capture.sh` lines 14-18). **The sidecar path needs DQ #302 resolved** (whether the kind is registered in `docs/brehon-law-inspired-network/` product-doc tree or `.claude/refs/` harness-local) but the JSONL file location itself is **`.claude/governance-log/retro-bypass.jsonl`** regardless. **Record fields** (per **DQ #297** advisor-resolved — `last_assistant_message_hash` was structurally wrong because `retro-check.sh` line 20 only reads `CLAUDE_PROMPT`, not the last assistant message; renamed): `timestamp` (ISO 8601 UTC), `session_id`, `attempt_count`, `prompt_hash` (SHA-256 of `CLAUDE_PROMPT` truncated to 16 hex chars — same prompt firing N consecutive Stop attempts is the suspect-loop signal), `branch_at_fail_open` (per `git rev-parse --abbrev-ref HEAD`), `kind: "retro_bypass"`.

8. **Governance-log schema registration** — the planner adds the `retro_bypass` kind to the governance-log schema document. Path: the planner reads `docs/brehon-law-inspired-network/04-data-model-and-api.md` (or the canonical kind-registry doc; planner reads the current file and locates the right section) and appends the entry with: kind name, JSONL schema, originating hook (`retro-check.sh` fail-open path), consumer (`weekly-review` Step 3 + future audit reads). Cite `feedback_lesson_must_pair_with_structural_fix_when_fixable.md` — the lesson saying "documentary guard without enforcement is a re-fire risk" applies here too; the schema registration is the structural fix that pairs with the new kind.

9. **New lesson `feedback_retro_bypass_governance_log.md`** — cross-links the instrumentation. Body: (a) the bypass class (3-attempt fail-open is by design for true loops, but cannot be invisible); (b) the structural fix (JSONL trail + weekly-review consumption); (c) the audit signal (rate of `retro_bypass` entries per week should be monotonically decreasing per RLS-PMD review §5.2 autonomy-readiness criterion 5.2); (d) cross-link `[[feedback_pmd_cross_lane_canonical_db]]`, `[[feedback_mcp_canonical_pmd_path_enforce_at_session_start]]`, `[[project_phase6_convention_divergence_class]]` (the conformance-audit's autonomy-readiness sibling). Per `feedback_one_system_memory_in_repo.md` + `feedback_lesson_mirror_check.md`.

**Track D — mandatory closeout:**

10. **Dogfood report** — `.claude/PRPs/reports/v1-rls-r1-dogfood-<YYYY-MM-DD>.md` per PRECON-8. Three dogfood runs:
    - **4.1 dogfood:** `pmd-canonical-guard.sh` against (a) THIS lane's `.mcp.json` (must exit 0 silent), (b) a `/tmp/sentinel.mcp.json` deliberately set to a wrong `PROJECT_MEMORY_DB` (must exit 0 with stderr WARN naming both paths). Capture stdout + stderr per run.
    - **4.6 dogfood:** weekly-review Step 3 against `.claude/PRPs/reports/*.md` for the last 30 days. Confirm the cycle-count-≥3 proposal from `feedback_plan_stub_uniformity_with_canonical_sibling.md` is surfaced. Capture the `.claude/harvest/<iso-week>.md` output.
    - **4.7 dogfood:** synthetic 3-attempt fail-open trigger; confirm a `retro_bypass` JSONL entry lands; confirm the schema doc lists the new kind. Capture the JSONL entry verbatim.

11. **Wire into the advisor stage-shape** — edit `.claude/rules/advisor-orchestrator.md`:
    - **§1 "Polling loop" addendum**: a one-line note that the SessionStart canonical-PMD guard is now active (`pmd-canonical-guard.sh` runs at every session start in lanes wired per the bootstrap checklist) — surfaces as a WARN on lane drift, not a block. Stage-shape unchanged.
    - **§5 "Validation, classification, recovery"**: add a new sub-section §5.5 (or wherever the canonical ordering places it) titled "Retro-bypass observability" describing the `retro_bypass` JSONL trail + weekly-review consumption + the autonomy-readiness criterion (bypass rate monotonically decreasing).

12. **Paired lesson files** — per `feedback_one_system_memory_in_repo.md` + `feedback_lesson_mirror_check.md` + `feedback_lesson_must_pair_with_structural_fix_when_fixable.md` (the structural fix IS the script + the rule file + the instrumentation — the lessons cross-link them):
    - `feedback_pmd_canonical_guard_enforces_invariant.md` — the v1-rls-r1 closure of `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`'s PENDING status. Body: pre-r1 the guard was documentary; post-r1 the script enforces; the SessionStart wiring is per-lane in `settings.local.json` per the updated bootstrap checklist. Cross-link `[[pmd-invariants]]` (the new rule file) and the closure of the PENDING status in the prior lesson.
    - `feedback_retro_bypass_governance_log.md` (per Track C Task 9 above — same file; same lesson).
    - (Optional) `feedback_retro_harvest_weekly_cadence.md` — the v1-rls-r1 closure of the retro-harvest ad-hoc gap. Body: pre-r1 retro-harvest skill was read-only; post-r1 weekly-review Step 3 runs it on Sunday cadence; one weekly artifact at `.claude/harvest/<iso-week>.md`. The planner decides between (a) one new lesson AND an edit to an existing one OR (b) edit-only to the existing `feedback_retro_not_report.md`/`feedback_four_role_retro_signals.md`; PRECON-9 lean: option (a) — new lesson is the durable record.

13. **Standard final §13 retro task** per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`.

### 2.2 Scope boundary — what is explicitly NOT in this sub-phase

Per the 2026-05-20 user decisions §0.1.1–0.1.9. The plan's §12 "NOT building" MUST enumerate these (each with one-line "why excluded" rationale and the future trigger if any):

- **No actual `MCPs/project-memory-mcp/dist/index.js` patch** (PRECON-2). The 4.2-spec doc is the contract; the patch ships separately. Trigger: a focused `MCPs/project-memory-mcp` PR after v1-rls-r1 ships the spec — may carry as v1-rls-r2 or as out-of-Brehon-cadence meta-tooling.
- **No Rust best-practices research items** (PRECON-3): `ENABLE_PROMPT_CACHING_1H=1` rollout, cargo-nextest, CodeRabbit CLI. Triggers: separate token-economy / tooling-adoption lanes; rust-analyzer-MCP install + Clippy disallowed_methods already in `brehon-conformance-audit` plan.
- **No items 4.3, 4.4, 4.5, 4.9, 4.10** (PRECON-5). Deferred to `v1-rls-r2`. Each with one-line rationale:
    - 4.3 `.mcp.json` documentary-guard re-read hook — adjacent to 4.1; lower leverage; bundle into v1-rls-r2.
    - 4.4 Junior post-task-retro inline-backfill PostToolUse hook — collapses if 4.2 patch ships; preferred path is 4.2 first.
    - 4.5 `sync-lessons-to-pmd.sh` auto-invoke — bundle into v1-rls-r2 weekly-review extensions.
    - 4.9 auto-phase 10-category mandatory section — orthogonal to the autonomy-substrate framing; bundle into a session-retro-skill lane.
    - 4.10 observation-capture.sh consumer — bundle into v1-rls-r2 weekly-review Step 4.
- **No new ADR.** v1-rls-r1 enforces existing invariants; no new design decision. The planner files `kind: "blocker"` if a new ADR appears necessary.
- **No auto-promote of harvest-surface proposals to `.claude/lessons/`.** Per the RLS-PMD review §4.6 contract — manual review thereafter; the skill SURFACES, the human PROMOTES.
- **No edit to `retro-check.sh`'s fail-open behaviour itself.** The 3-attempt cap stays (loops are real). The instrumentation is additive. Per RLS-PMD review §4.7.
- **No auto-wiring of `pmd-canonical-guard.sh` in lanes other than the current one** (PRECON-9). Per-lane bootstrap-checklist update is the propagation mechanism.
- **No edit to MCP server source from this repo.** PRECON-2 hard refusal.
- **No `webauthn-rs` step-up gate** (no write surface beyond what already gates `.claude/`).
- **No subagent variant of the canonical-guard.** The hook is inline-invoked at SessionStart; subagent variant is anti-pattern per cost model.
- **No auto-trigger keyword-stuffing** in any skill/rule `description:` field. Per `feedback_principles_not_rules.md` + the conformance-audit-PRECON-3 precedent.
- **No `cargo` invocation from any v1-rls-r1 ship surface.** §15 cargo runs are sanity-only on the existing wrapper scripts; no script/hook/skill in v1-rls-r1 invokes cargo at runtime.

**Hard out-of-scope (per the broader v1 Brehon platform):** auto-apply (v3 / ADR-006), reputation portability (v2/v3), cross-instance jury (v3), OPA federation policy (v2). v1-rls-r1 hardens the *harness*; it does NOT change Brehon governance behaviour.

### 2.3 Open questions — clarify-gate inputs

The 2026-05-20 user decisions resolved the major open questions (PRECON-1 through PRECON-9). The /brehon-clarify pass at 2026-05-20 advisor-resolved DQ #296-#300 (mechanical / cite-able ambiguities) and surfaced DQ #301-#302 to the user (judgment-heavy: durable-artifact placement, design-doc shape). Residual planner-time choices (no clarify-DQ needed — the planner makes the call at plan time per `feedback_principles_not_rules.md`):

1. **CLOSED — Governance-log emission path for `retro_bypass`** (was: candidate sidecar JSONL vs observation-capture.sh). Resolved by structural finding: `.claude/hooks/observation-capture.sh` lines 14-18 write to `~/.cache/tw-observations/<PPID>.jsonl` — gitignored, per-PPID, transient, shadow-mode runtime cache — NOT a durable trail. **Sidecar JSONL at `.claude/governance-log/retro-bypass.jsonl`** is the binding path (the JSONL file location; whether the kind is **registered** in the product-doc tree is the OPEN DQ #302). The §13 task body says "write JSONL to `.claude/governance-log/retro-bypass.jsonl`" verbatim. Per `.claude/hooks/observation-capture.sh` lines 14-18.

2. **`.claude/harvest/` gitignored vs tracked.** Per Track B Task 5 — the planner lean per PRECON-9 NOT-binding is gitignored (per the `.claude/PRPs/audit-metrics/` analogue from the conformance-audit plan). If tracked, the planner adds a 4-week-retention sweep step to weekly-review Step 1. **Planner-time choice; no clarify-DQ.**

3. **Whether the new `pmd-invariants.md` rule loads at SessionStart.** Per `.claude/rules/` convention — all rules with no `paths:` frontmatter auto-load at SessionStart. The planner decides whether to add a `paths:` frontmatter (scoping the load) OR leave it auto-loading. Lean (NOT binding): leave it auto-loading — five invariants on a single page, ~150 lines max, fits within MEMORY.md's startup-budget calculus. **Planner-time choice; no clarify-DQ unless size grows.**

4. **CLOSED — `last_assistant_message_hash` field for `retro_bypass`** (was: candidate `last_assistant_message_hash` vs deriving from hook input). Resolved by **DQ #297** (advisor-resolved): the hook only sees `CLAUDE_PROMPT` (line 20), not the last assistant message — so the field is renamed to `prompt_hash` (SHA-256 of `CLAUDE_PROMPT` truncated to 16 hex chars). Same loop-detection signal value; structurally accessible to the hook. The §13 Track C Task 7 task body lists `prompt_hash` (not `last_assistant_message_hash`). Per `.claude/hooks/retro-check.sh` line 20 + DQ #297.

5. **MEMORY.md entry for the new pmd-invariants rule.** The 5-line addition under "Promoted patterns" or "Active workflow state". The planner picks the slot; lean: under "Promoted patterns" since the rule consolidates 5+ pattern occurrences (the criterion from the existing memory-injection / lesson-promotion discipline). **Planner-time choice; no clarify-DQ unless MEMORY.md is near the 200-line cap.**

6. **OPEN — one-shot SessionStart wiring location** (per **DQ #301** user-relay-pending). Where does the one-shot `pmd-canonical-guard.sh` SessionStart wiring happen: this lane (current PRECON-9; ephemeral), canonical `brehon-fork` only (durable; loses lane-dogfood), both (cheap; redundant), or defer to bootstrap-only? The user picks at brief-approval. The planner's PRECON-9 wording stays unchanged until DQ #301 resolves; the planner's task body for Track B Task 6 cites DQ #301 as the binding source.

7. **OPEN — `retro_bypass` schema-doc placement** (per **DQ #302** user-relay-pending). Where to register the new JSONL kind: extend `docs/brehon-law-inspired-network/04-data-model-and-api.md` (mixes product + harness; sub-section split), new file `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md` (clean split; ADR-adjacent), `.claude/refs/governance-log-jsonl-kinds.md` (harness-internal scope), or no separate doc (schema lives in the new lesson body). The user picks at brief-approval. The planner's Track C Task 8 body cites DQ #302 as the binding source.

The advisor (not the planner) files `kind: "clarify"` DQs at the clarify-gate. Planner ambiguities are `kind: "blocker"` (for genuine open decisions) or `kind: "log"` (for choices the planner has made with rationale). Per `.claude/rules/decision-queue.md` hard refusals #6.

### 2.4 Plan file deliverable shape

- **One file:** `.claude/PRPs/plans/v1-rls-r1.plan.md`.
- **Follow `.claude/PRPs/templates/plan.template.md` 20-section schema literally.** §16a Stories mandatory.
- **PRECON-7 DoD shape — `validate-pending-laptop`, NOT Shape-G.** §15 names the cargo DoD commands verbatim with `--workspace --features full`. Mirror `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` §15 as the laptop-shape exemplar.
- **§15 expected commands** (the planner refines exact incantations against current `.gitignore` + scripts):
    - `bash scripts/brehon/cargo-check.sh --workspace --features full` — confirms no incidental breakage from `.claude/` / `docs/` edits.
    - `bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings` — confirms no incidental clippy regression.
    - `cargo test --test e2e --no-run -p lemmy_server` — phase-branch test-link still works.
    - `bash .claude/hooks/pmd-canonical-guard.sh` (probe-run) — confirms the new hook script behaves per §0.1.8 (exit 0 silent on canonical; WARN+exit 0 on deliberate-mispoint sentinel).
    - `bash .claude/hooks/retro-check.sh` (cannot synthetically trigger fail-open without a 3-attempt stub; the dogfood task per Track D Task 10 covers this via a separate probe path the planner names).
- **§5 complexity-factor breakdown table** per `feedback_complexity_score_pre_split.md`. **Pre-estimate: ~6-8** — moderate, contained, mostly documentation + thin shell scripts:
    - §13 impl tasks above 5: +2-3 (Tasks 1-13 above ≈ 13 tasks; the planner's exact count drives the +1/+2 increment; SOME tasks are very small — a one-line `.gitignore` add — so the count vs blast-radius weighting per the template's §5 formula matters).
    - Migrations: 0.
    - Crates touched: 0.
    - e2e edits: 0.
    - ADR-affecting: 0.
    - Cargo budget: 0 (no Rust changes).
    - **Estimated score: ~6-8**, comfortably below Sonnet's split threshold of 8. If the score crosses 8, the natural SEAM is Track A vs Track B vs Track C — Track A first (doc + spec; lowest blast radius), Track B second (hooks + skill edits), Track C third (`retro-check.sh` instrumentation + governance-log schema). The planner proposes the split in a `kind: "blocker"` DQ if needed; the advisor resolves at plan-approval.
- **§13 per-task `creates:` / `modifies:` FILES YAML block** mandatory (load-bearing for cohort dispatch + the §5 complexity heuristic). `[P]` markers only where FILES-YAML disjointness genuinely holds. **Critical dependency structure for `[P]`:**
    - **Track A Tasks 1 (mcp-write-time-embedding.md), 2 (pmd-invariants.md)** — disjoint files; `[P]` candidates. Both `requires: [0]` (Task 0 pre-flight only).
    - **Track B Task 3 (pmd-canonical-guard.sh)** — disjoint from Track A; `[P]` with Tasks 1-2 candidate. `requires: [0]`.
    - **Track B Task 4 (weekly-review SKILL.md Step 3)** — disjoint from Tasks 1-3; `[P]` candidate. `requires: [0]`.
    - **Track B Task 5 (.gitignore + harvest dir)** — depends on Task 4 (the `.gitignore` entry references the harvest dir Task 4 introduces). `requires: [4]`. Not `[P]`.
    - **Track B Task 6 (bootstrap-checklist update + this-lane wiring)** — depends on Task 3 (the script Task 3 ships is what the checklist entry wires). `requires: [3]`. Not `[P]`.
    - **Track C Task 7 (retro-check.sh fail-open instrumentation)** — disjoint from Tracks A/B; `[P]` candidate with Tasks 1-4. `requires: [0]`.
    - **Track C Task 8 (governance-log schema registration)** — depends on Task 7 (the kind name + JSONL schema cite Task 7's emit format). `requires: [7]`. Not `[P]`.
    - **Track C Task 9 (new lesson feedback_retro_bypass_governance_log.md)** — depends on Tasks 7+8 (lesson cites the instrumentation + the schema kind). `requires: [7, 8]`. Not `[P]`.
    - **Track D Task 10 (dogfood report)** — depends on ALL prior. `requires: [3, 4, 5, 6, 7, 8, 9]`. Serial.
    - **Track D Task 11 (advisor-orchestrator.md wiring)** — depends on Tasks 3 + 7 (the rule edits cite both). `requires: [3, 7]`. Not `[P]` with Task 10.
    - **Track D Task 12 (paired lesson files)** — depends on Tasks 3 + 7 + 10. `requires: [3, 7, 10]`. Two lesson files are disjoint; `[P]` between themselves; not `[P]` with Tasks 11 or 13.
    - **Track D Task 13 (retro)** — last. Serial.

    The planner sets `requires:` arrays on every task with a cross-task dependency (per `.claude/rules/advisor-orchestrator.md` §4.1 step 4a — load-bearing to prevent the Cohort-A/Cohort-B isolation bug class).

- **§16a Stories** — every story names composing §13 tasks + the (laptop-mode) DoD checkpoint + Brief-Scope outputs. Likely **5 stories**:
    - **Story 1 — Track A artifacts shipped**: `mcp-write-time-embedding.md` + `pmd-invariants.md` exist, frontmatter valid, MEMORY.md entry added.
    - **Story 2 — SessionStart guard wired in this lane**: `pmd-canonical-guard.sh` script exists, executable, dogfooded (zero false positive on canonical; loud WARN on deliberate-mispoint); `settings.local.json` entry added in this lane; bootstrap-checklist lesson updated.
    - **Story 3 — weekly-review Step 3 + harvest dir**: SKILL.md edited; `.claude/harvest/` directory pattern established (gitignored per PRECON-9 lean unless clarify-DQ resolves otherwise); dogfood produces `.claude/harvest/<iso-week>.md` artifact.
    - **Story 4 — `retro_bypass` instrumentation**: `retro-check.sh` fail-open emits JSONL; governance-log schema doc registers the new kind; new lesson cross-links; dogfood synthetic-3-attempt trigger produces a verifiable JSONL entry.
    - **Story 5 — wiring + lessons + retro shipped**: advisor-orchestrator.md edited; paired lesson files exist and cross-link correctly; retro file authored per `feedback_retro_not_report.md`.

- **§4 watchpoints** — every entry cites a SPECIFIC file:line / function name / table at CURRENT HEAD (per `feedback_advisor_watchpoint_specificity.md`). The 6-watchpoint seed list is in §4.1 below.

- **§6 "Relationship to other v1 sub-phases"** — confirm: v1-rls-r1 is **harness-level**, independent of any active impl sub-phase. It does not block `v1-federation-inbound-c` (the next federation sub-phase) nor `brehon-conformance-audit` (the parallel meta-skill sub-phase). v1-rls-r1's first production beneficiaries are: (a) the next lane bootstrap (the `pmd-canonical-guard.sh` activates per the updated checklist); (b) the next Sunday weekly-review (Step 3 fires); (c) any future retro-check fail-open (the JSONL trail begins immediately on ship).

**Commit only the plan file** (and any planner DQ entries — pushed immediately per `.claude/rules/decision-queue.md` "Mid-task visibility"). Plan-file commit pushes to `phase-v1-rls-r1` after the advisor's DoD smoke + watchpoint-specificity gates pass and the user approves (User Gate 1).

---

## 3. Required reading (in order, before drafting any plan section)

1. **This brief in full** — the substantive context + the PRECON decisions.
2. `.claude/agents/planning.md` — the planning subagent contract (model-enforcement, §13 FILES YAML, §5 split threshold, §16a Stories, hard refusals).
3. `.claude/PRPs/templates/plan.template.md` — canonical 20-section plan schema.
4. **`docs/research/brehon-rls-pmd-review.md`** — the RLS-PMD review this brief absorbs. Read in full; specifically:
    - §1 (Layered architecture — the five retro tiers): the substrate v1-rls-r1 hardens.
    - §2 (Retro flow trace — observation to canonical pattern): the eight-stage handoff with five "load-bearing on honesty" arrows; v1-rls-r1 converts three of those to enforcement.
    - §3 (PMD write/embed/search pipeline — the two-system distinction): the substrate for items 4.2-spec and 4.8.
    - §4.1 (SessionStart canonical-PMD guard): the spec for Track B Task 3.
    - §4.2 (No write-time embedding): the spec for Track A Task 1.
    - §4.6 (retro-harvest is read-only, ad-hoc): the spec for Track B Task 4.
    - §4.7 (retro-check.sh fail-open): the spec for Track C Task 7.
    - §4.8 (five PMD-meta lessons should promote to a top-level rule): the spec for Track A Task 2.
    - §5 (Autonomy-readiness criteria 1-5): the why-this-matters framing for §6 of the plan.
    - §6 (Top 5 highest-leverage hardenings): the binding scope list.
5. **`.claude/lessons/feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`** — the existing spec for Track B Task 3 (item 4.1). Read in full; lift the mechanism verbatim. This lesson IS the spec.
6. **`.claude/lessons/feedback_pmd_cross_lane_canonical_db.md`** — the originating incident + the diagnosis recipe Track B Task 3 automates.
7. **`.claude/lessons/feedback_pmd_backfill_after_write.md`** — the originating incident + the manual mitigation for Track A Task 1 (item 4.2-spec) to obsolete.
8. **`.claude/lessons/feedback_pmd_two_memory_systems_distinction.md`** — the System 1 / System 2 distinction Track A Task 2 invariant #2 codifies.
9. **`.claude/lessons/feedback_lesson_must_pair_with_structural_fix_when_fixable.md`** — the meta-pattern v1-rls-r1 enforces (every documentary guard pairs with shipped enforcement OR a `fix status: PENDING` lesson).
10. **`.claude/lessons/feedback_principles_not_rules.md`** — the WARN-vs-FAIL discipline + the anti-keyword-stuffing discipline for `description:` fields.
11. **`.claude/lessons/feedback_one_system_memory_in_repo.md`** + `.claude/lessons/feedback_lesson_mirror_check.md` — the lesson-mirror discipline for Track D Task 12.
12. **`.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md`** — Track B Task 6 EDITS this lesson; read in full to identify the right insertion point.
13. **`.claude/lessons/feedback_settings_local_json_worktree_bootstrap.md`** — the per-worktree + gitignored discipline Track B Task 6 wires under.
14. **`.claude/lessons/feedback_retro_not_report.md`** + `.claude/lessons/feedback_four_role_retro_signals.md` + `.claude/lessons/feedback_retro_task_complexity_score.md` — Track D Task 13 (retro) authoring discipline.
15. **`.claude/skills/weekly-review/SKILL.md`** — the file Track B Task 4 EDITS; read in full to identify the right insertion point (between existing Step 2 promotion and the current Step 3 lesson-clustering — the new harvest step likely renumbers existing steps; the planner picks).
16. **`.claude/skills/retro-harvest/SKILL.md`** — the existing read-only skill Track B Task 4 CONSUMES. Read in full; the new weekly-review Step 3 invokes the harvest skill's logic OR duplicates it inline; the planner picks (lean per PRECON-9 NOT-binding: invoke-by-reference — `Skill` tool call inside Step 3 — but the planner verifies the skill is invocable from another skill).
17. **`.claude/skills/session-retro/SKILL.md`** + `.claude/skills/post-task-retro/SKILL.md` — context for the retro tier `pmd-canonical-guard.sh` protects.
18. **`.claude/hooks/retro-check.sh`** — Track C Task 7 EDITS this file; read in full to identify the fail-open path (the 3-attempts-then-exit-0 branch per RLS-PMD review §3) and the JSON input contract (per Anthropic hook reference).
19. **`.claude/hooks/observation-capture.sh`** — context for Clarify-DQ candidate §2.3 #1 (the structured-stderr alternative to a sidecar JSONL); read enough to confirm whether `retro_bypass` belongs in shadow-capture or in its own sidecar.
20. **`.claude/hooks/pre-phase-audit.sh`** — context only; per the RLS-PMD review §4.1 the canonical-guard sequences BEFORE pre-phase-audit at SessionStart. The planner's `settings.local.json` snippet for Track B Task 6 sequences the two correctly.
21. **`.claude/rules/multi-lane-worktree.md`** — §"PMD is cross-lane shared" + §"Worktree-aware DQ id discipline" + the canonical-checkout vs lane-dedicated split. v1-rls-r1's enforcement converges on the canonical PMD via the canonical-path invariant Track A Task 2 codifies.
22. **`.claude/rules/advisor-orchestrator.md`** — §3.1 stage-shape (Track D Task 11 edits this section), §3.4 DoD smoke gate, §3.5 watchpoint-specificity gate, §3.7 dogfood gate (Task 10 satisfies this), §3.8 schema-changing-spec retrofit gate (the planner asks `AskUserQuestion` at the §3.8 trigger if needed — this plan adds NEW artifact classes (`.claude/PRPs/specs/`, `.claude/harvest/`, `.claude/hooks/pmd-canonical-guard.sh`) and a NEW section in `advisor-orchestrator.md`; per §3.8 "purely additive functionality" is a skip condition — the planner confirms NO existing artifact-class shape changes), §3.9 verify gate, §4.1 cohort dispatch (the planner sets `requires:` arrays rigorously), §5 validation/classification/recovery (Track C extends with §5.5).
23. **`.claude/rules/decision-queue.md`** — schema-v2 (Tasks 1+ may file `kind: "blocker"` and `kind: "log"` entries; the dogfood Track D Task 10 may write `kind: "log"` to record dogfood findings).
24. **`.claude/rules/branch-manager.md`** — file ownership boundaries (the BM Junior task that opens the PR for this sub-phase MUST NOT touch the script content; only PR title + body + runlog entries).
25. **`.claude/rules/pmd-search-strategy.md`** — Track A Task 1 + 2's `description:` fields are hybrid-search-friendly natural-language sentences describing WHAT the artifact does (not auto-trigger keywords); the planner phrases them per this rule.
26. **`.claude/PRPs/reports/session-retro-2026-05-18-pmd-stranding-remediation.md`** — the v1-ship-1 incident retro that informs PRECON-1 (item 4.1's PENDING status closure rationale) + PRECON-9 (the bootstrap-checklist propagation mechanism).
27. **`.claude/PRPs/plans/v1-federation-inbound-a.plan.md`** — the structural-skeleton + discipline mirror for §1-§4/§5/§6/§15-laptop-DoD/§16a/§19. Read in full per `feedback_read_canonical_before_writing_spec.md` + `.claude/rules/advisor-orchestrator.md` §3.6 (canonical-schema-first gate).
28. **`.claude/PRPs/briefs/brehon-conformance-audit-planning-1.md`** — the parallel meta-skill brief whose shape this brief mirrors (eight-section structure, PRECON sub-section, Track-A/B/C split). Read in full per `feedback_read_canonical_before_writing_spec.md`.

The §2.4 file-class mandatory-lesson injection (per `.claude/rules/advisor-orchestrator.md` §2.4) does NOT auto-fire on this PLANNING brief (the brief authors a plan, not impl code). The §2.4 table WILL auto-fire when the advisor later authors the IMPL briefs from this plan — the plan should make that injection mechanical by naming the file classes per task in the FILES YAML block. Since v1-rls-r1 touches no `crates/**` / no `migrations/` / no e2e, the table's relevant rows for impl-brief authoring are: `.gitignore` add (Track B Task 5 — fires `feedback_settings_local_json_worktree_bootstrap.md` if applicable) and `.claude/hooks/*.sh` edits (Track B Task 3 + Track C Task 7 — no current §2.4 row; the planner may propose adding one in a `kind: "log"` DQ if the dogfood surfaces a recurring failure mode worth codifying).

---

## 4. Constraints — what the planner MUST enforce

### 4.1 Seed watchpoint list (the planner extends with file:line citations at plan time)

| # | Watchpoint | What to watch | Where to cite (current HEAD `6e16ee94f`) |
|---|---|---|---|
| 1 | **The hook scripts exit 0 always (WARN-not-FAIL).** Per `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` §"Why a SessionStart WARN, not a PreToolUse block or a hard fail". Any task that writes a hook script with `exit 1` on a config-error path is a discipline breach. | Track B Task 3 (pmd-canonical-guard.sh) + Track C Task 7 (retro-check.sh additions) — both must exit 0 with stderr WARN. | `.claude/hooks/pmd-canonical-guard.sh` (Track B Task 3 output); `.claude/hooks/retro-check.sh` (Track C Task 7 modifies). |
| 2 | **The five PMD invariants are FIXED.** Per PRECON-5. Track A Task 2 ships exactly five (canonical path / two-systems / no-write-time-embedding / LESSON-trailer / SessionStart guard). No sixth, no fourth. Candidate additions go through a future v1-rls-r2 or via lesson promotion. | Track A Task 2's `pmd-invariants.md` enumerates exactly 5. | `docs/research/brehon-rls-pmd-review.md` §4.8 — the source list. |
| 3 | **No cargo invocation from any v1-rls-r1 script/hook/skill.** Per PRECON-7. The §15 cargo runs are sanity-only; the deliverables themselves never invoke cargo. | Every task's body declares this in its description; the dogfood Task 10 verifies. | `pattern_cargo_feature_flag_propagation.md` PMD pattern + `project_shape_g_suspended_2026_05_16` PMD project memory. |
| 4 | **Mid-task DQ push discipline** (applies at BOTH stages — planning AND impl, per **DQ #300** advisor-resolved). Per `.claude/rules/decision-queue.md` "Mid-task visibility" + "Subagents and attribution" — every DQ entry the planning Junior task files during plan authoring OR the impl-task workers file during impl execution MUST be committed AND pushed to the worker branch immediately. The advisor's `git fetch` picks it up. | Every task's body declares mid-task push; per-cohort fan-out timing; planning Junior's `kind: "blocker"` / `kind: "log"` DQs (per §4.2) also push mid-task. | `.claude/rules/decision-queue.md` Recipe 1-2-3 + "Subagents and attribution". |
| 5 | **No auto-trigger keyword-stuffing.** Per PRECON-3 + `feedback_principles_not_rules.md`. The `description:` field on `pmd-invariants.md` frontmatter + `pmd-canonical-guard.sh` header comment + new lesson file frontmatter are PLAIN-LANGUAGE one-sentence descriptions. No "MUST USE THIS" / "CRITICAL" / "ALWAYS INVOKE" verbiage. | Frontmatter review at plan-approval. | Track A Task 2 (rule file), Track B Task 3 (hook script header), Track D Task 12 (lesson files). |
| 6 | **The dogfood (Task 10) is the integration test.** Per PRECON-8. Do NOT dogfood before Tracks A/B/C land and §15-green; do NOT skip the dogfood. Three sub-runs: 4.1 hook against this-lane + sentinel; 4.6 weekly-review Step 3 against 30-day retro corpus; 4.7 synthetic 3-attempt fail-open. | Task 10 `requires: [3, 4, 5, 6, 7, 8, 9]`. | `.claude/PRPs/reports/v1-rls-r1-dogfood-<date>.md`. |
| 7 | **`retro-check.sh` fail-open behaviour is PRESERVED.** Per PRECON-2 (review §4.7). Track C Task 7 INSTRUMENTS the fail-open with a JSONL trail; it does NOT change the 3-attempt cap, the fail-open exit code, or any other Stop-hook semantic. Loops are real; the hook must permit them. | Track C Task 7's diff against `retro-check.sh` is ADDITIVE only (function appended; existing fail-open branch unchanged except for one function-invoke line). | `.claude/hooks/retro-check.sh` at HEAD — the planner reads + cites the fail-open block's existing exit-code line. |
| 8 | **Documentary guards pair with shipped enforcement (per `feedback_lesson_must_pair_with_structural_fix_when_fixable.md`).** Every `pmd-invariants.md` invariant cites the SHIPPED enforcement (script/hook/skill that protects it) OR explicitly carries `enforcement: PENDING` with a future-trigger note. v1-rls-r1's whole motivation is closing these PENDING gaps; no new PENDING items introduced unless explicitly named in §12 "NOT building". | Track A Task 2's rule file body — each invariant's "How to apply" line either names a SHIPPED enforcement OR carries `enforcement: PENDING + see v1-rls-r2`. | The lesson + the new rule file. |

The planner expands this list with file:line citations at plan-time (per `feedback_advisor_watchpoint_specificity.md` — concept-only watchpoints are insufficient; every watchpoint cites a SPECIFIC table, file, function, or schema.rs line).

### 4.2 Behavioural constraints

- **The planning subagent NEVER writes the contents of `pmd-invariants.md`, `pmd-canonical-guard.sh`, the weekly-review Step 3 body, the `retro-check.sh` function append, the spec doc, the lesson files, the dogfood report.** Those are impl-task deliverables, NOT plan deliverables. The plan SPECIFIES what each artifact contains (the contract) but does NOT author the artifact. Per `.claude/agents/planning.md`.
- **The planning subagent NEVER writes `kind: "clarify"` DQ entries.** Per `.claude/rules/decision-queue.md` hard refusal #6. Planner ambiguities are `kind: "blocker"` or `kind: "log"`.
- **Attribution integrity per `.claude/rules/decision-queue.md`** — planner DQ entries use `from: "planner"`. The planner NEVER writes `answered_by: "advisor"` (a hard refusal).
- **Pre-push cargo discipline per `feedback_fix_impl_pre_push_cargo_check.md`** — although v1-rls-r1 touches no Rust, the plan's §13 task instructions for Tracks B+C SHOULD require the impl-task worker to run `bash scripts/brehon/cargo-check.sh --workspace --features full` locally before pushing the worker branch IF the worker is on Windows (laptop) and IF any `.claude/settings*.json` edit could conceivably touch a permission or hook that affects cargo invocation paths. The planner decides per task; lean: include in all Track B+C tasks as a 30-second pre-push gate.
- **Canonical-sibling-mirror discipline per `feedback_read_canonical_before_writing_spec.md`** — the plan's §13 task instructions MUST require any worker authoring a new artifact (script / rule / skill section / lesson) to FIRST Read 1-2 existing canonical sibling artifacts. Specifically: Track A Task 1 reads existing `.claude/PRPs/specs/` (if any) OR `docs/research/brehon-rls-pmd-review.md` §4.2 as canonical; Track A Task 2 reads existing `.claude/rules/*.md` (especially `decision-queue.md` for tone + structure); Track B Task 3 reads `.claude/hooks/retro-check.sh` + `.claude/hooks/pre-phase-audit.sh` as canonical hook shape; Track B Task 4 reads existing `.claude/skills/weekly-review/SKILL.md` step 1+2 as canonical step-style; Track C Task 7 reads `.claude/hooks/retro-check.sh` end-to-end before append; Track D Task 9+12 read existing `.claude/lessons/feedback_*.md` files (especially those cited in §3 above) as canonical lesson shape. The plan §13 task descriptions cite this requirement.

### 4.3 Hard refusals (the planner MUST stop on these)

1. **NEVER propose a §13 task that ships the actual `MCPs/project-memory-mcp/dist/index.js` patch.** PRECON-2 is binding. Track A Task 1 ships the SPEC only.
2. **NEVER propose a §13 task that adds a sixth (or removes a fifth) PMD invariant in `pmd-invariants.md`.** PRECON-5 is binding.
3. **NEVER propose a §13 task that changes `retro-check.sh`'s fail-open behaviour beyond instrumentation.** PRECON-2 / Watchpoint #7. The 3-attempt cap stays.
4. **NEVER propose a §13 task that adds a Rust-best-practices item.** PRECON-3 is binding.
5. **NEVER propose a §13 task that auto-wires `pmd-canonical-guard.sh` in lanes other than the current one.** PRECON-9 — per-lane bootstrap-checklist propagation is the mechanism.
6. **NEVER propose a §13 task that converts the canonical-guard hook into a subagent OR auto-trigger.** Per PRECON-3 anti-pattern + `feedback_principles_not_rules.md`.
7. **NEVER propose a §13 task that touches `crates/**`, `migrations/`, `Cargo.toml`, `Cargo.lock`, or `rust-toolchain.toml`.** v1-rls-r1 is harness-only.
8. **NEVER propose a §13 task that re-runs §15 cargo to produce ground truth.** Ground truth here is shell-exit-code of `pmd-canonical-guard.sh` + presence/absence of JSONL records — NOT cargo output.
9. **NEVER propose a §13 task that auto-promotes harvest-surface proposals to `.claude/lessons/` or `CLAUDE.md`.** Per RLS-PMD review §4.6 — manual review thereafter; the skill SURFACES, the human PROMOTES.
10. **NEVER propose adding the v1-rls-r1 deliverables to ANY auto-loaded chain beyond the explicit `.claude/rules/pmd-invariants.md` rule load and the `.claude/settings.local.json` SessionStart entry.** No CLAUDE.md import; no agent-definition import; no hook chaining beyond the SessionStart entry. Per PRECON-3.
11. **NEVER propose a §13 task that omits the dogfood (Task 10).** Watchpoint #6 — dogfood IS the integration test; without it, the deliverables are unverified.

If the planner sees a contradiction between this brief and any other source (PRD, ADR, prior plan, lesson, the RLS-PMD review itself), STOP and file `kind: "blocker"`. Do NOT silently resolve.

---

## 5. Dogfood — pre-commit walkthrough (per `feedback_dogfood_slash_command_specs.md`)

This brief was dogfooded against the current `phase-v1-rls-r1` lane at HEAD `6e16ee94f` before commit, per `.claude/rules/advisor-orchestrator.md` §3.7 Dogfood gate (planning-stage class).

**What worked:**

- The Track A/B/C structural split cleanly separates the doc-only deliverables (Track A — zero compile dependency) from the behaviour-changing scripts (Tracks B+C). The `requires:` dependency graph is acyclic and supports `[P]` cohorts within each Track.
- The five PMD invariants in Track A Task 2 map 1:1 onto the five PMD-meta lessons named in the RLS-PMD review §4.8. Zero invariants fell outside the existing lesson corpus; no new invariant invented.
- The lesson `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` IS the spec for Track B Task 3 — the planner does not re-derive the mechanism; the lesson's §"Mechanism" section is the verbatim contract.
- The WARN-not-FAIL discipline (Watchpoint #1) consistently applies across all v1-rls-r1 hook surfaces. No hook script exits non-zero on a config-error path; all surface as stderr WARN + exit 0 per the lesson's WARN-vs-FAIL reasoning.
- The dogfood (PRECON-8 / Track D Task 10) gives each Track a verifiable integration test. The 4.1 dogfood uses the current lane as one input (zero false positive) + a sentinel as the other (verifiable WARN). The 4.6 dogfood uses the existing 30-day retro corpus (already on disk). The 4.7 dogfood uses a synthetic 3-attempt trigger (testable via env var or stub).

**What didn't (and how the brief responds):**

- The exact governance-log emission path for `retro_bypass` (sidecar JSONL vs structured stderr to observation-capture.sh) is not pre-decided. §2.3 ambiguity #1 surfaces this as a clarify-DQ candidate. The planner's lean (sidecar JSONL) is rationale'd but not binding; the integration with observation-capture is preserved as an alternative.
- The `.claude/harvest/` gitignored-vs-tracked decision is not pre-decided. §2.3 ambiguity #2 surfaces this as a clarify-DQ candidate. The lean (gitignored per the audit-metrics analogue) keeps weekly-review's output cadence-bound; tracked alternative is preserved.
- The MCP-patch-not-shipped-here decision (PRECON-2) means item 4.2's downstream-collapse claim (delete Step 5.5; weekly-review §1b becomes aspirational) is **forward-looking** — the spec doc names the claim but cannot verify it within this sub-phase. The planner notes this in the spec doc as "verification deferred to the MCPs/project-memory-mcp PR's own DoD".
- The bootstrap-checklist propagation for `pmd-canonical-guard.sh` SessionStart wiring in lanes other than the current one (PRECON-9) is **deferred** — currently-active lanes (canonical `brehon-fork`, `brehon-fork-conformance-audit`, `brehon-fork-tooling`) wire on their next bootstrap cycle. The planner notes this in §10 "Out of scope" with the rationale ("lane wiring is per-lane drift; the checklist catches it on next bootstrap; auto-wiring all active lanes is a one-off side quest not central to v1-rls-r1's mission").

The dogfood completed in ~15 min (walkthrough of the brief against the five RLS-PMD review items + the existing hook/skill/rule shape). Cost-benefit confirmed: ~15 min dogfood vs ~30-60 min remediation cost of a planning miss caught at plan-approval.

---

## 6. After the planner ships

1. **Junior planning task completes** — `.claude/PRPs/plans/v1-rls-r1.plan.md` is on `phase-v1-rls-r1` via Junior's finalize-merge.
2. **Advisor runs §3.4 DoD smoke test** — every command in plan §15 against current HEAD; the new `pmd-canonical-guard.sh` (if Track B Task 3 shipped early as part of the plan-Junior's authoring — unlikely but possible) runs against the lane; the `bash scripts/brehon/cargo-check.sh ...` confirms no incidental Rust breakage.
3. **Advisor runs §3.5 watchpoint-specificity gate** — every watchpoint in plan §4 cites a SPECIFIC file:line / function / table. Concept-only entries trigger a DQ for revision before plan approval.
4. **Advisor runs §3.7 dogfood gate** — confirms plan §1's pre-commit dogfood section is present.
5. **Advisor runs §3.8 schema-changing-spec retrofit gate** — this plan adds NEW artifact CLASSES (`.claude/PRPs/specs/`, `.claude/hooks/pmd-canonical-guard.sh`, `.claude/harvest/`, `.claude/rules/pmd-invariants.md`). It also adds a NEW section to `.claude/rules/advisor-orchestrator.md`. Per §3.8 "purely additive functionality" is a skip condition — the planner CONFIRMS in §10 that no existing artifact-class shape changes (e.g. no change to plan template; no new required section in existing skill files; only ADDITIVE deltas to `weekly-review/SKILL.md` + `retro-check.sh`). No retrofit needed.
6. **User Gate 1 (plan approval)** — advisor surfaces the plan + the four gate outcomes (DoD smoke / watchpoint specificity / dogfood / schema-retrofit) to the user. User approves or requests revisions.
7. **On approval — impl tasks dispatch** — per the plan §13 cohort structure. Tracks A/B/C respect their `requires:` graph; cohort dispatch goes through the §4.1 sequence with YAML overlap check + `requires:` dependency check + budget check (Shape-G suspended, so budget non-binding; per PRECON-7 this sub-phase ships zero new cargo budget).
8. **§15 laptop-mode validation per impl-task** — `validate-pending-laptop` DQ entries; advisor mutates per `.claude/rules/advisor-orchestrator.md` §5.2.
9. **Dogfood (Task 10) is the user-visible gate** — the dogfood report + harvest artifact + JSONL trail land on the phase branch; advisor confirms (a) `pmd-canonical-guard.sh` zero false-positive on canonical + WARN on sentinel, (b) weekly-review Step 3 surfaces ≥1 known harvest target, (c) synthetic-3-attempt fail-open produces a `retro_bypass` JSONL entry with all required fields.
10. **bm-pr → CR → triage → fix-in-PR → bm-merge** — standard Brehon pipeline per `.claude/rules/branch-manager.md` + the BM verb scripts. User Gates 3-5 fire normally.
11. **Retro task (Task 13)** — authored per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`. User Gate 6.
12. **`/brehon-phase-transition`** — closes the sub-phase, archives DQ entries per the archive policy at `.claude/rules/decision-queue.md`.

---

## 7. Definition-of-done for THIS planning brief

This brief is DONE when:

- This file at `.claude/PRPs/briefs/v1-rls-r1-planning-1.md` is committed to `phase-v1-rls-r1`.
- A clarify-gate pass has run (`/brehon-clarify .claude/PRPs/briefs/v1-rls-r1-planning-1.md`) and all clarify-DQs are resolved.
- The user has approved (or modified-then-approved) this brief's scope (User Gate 1-pre — brief-level approval, per PRECON-4).
- The Junior planning task is queued with the matching dispatch line in §1.

The plan file itself is the Junior planning task's deliverable; that completion is downstream of this brief.

---

## 8. Commit + push

This brief commits as: `chore(advisor): author v1-rls-r1 planning brief — RLS hardening Wave 1 (top-5 RLS-PMD review items)`

Per `.claude/rules/decision-queue.md` Attribution-integrity §Detection: the subject pattern matches `^(chore|docs)\((advisor|decision-queue)\)`.

Push to `phase-v1-rls-r1`. Then run `/brehon-clarify .claude/PRPs/briefs/v1-rls-r1-planning-1.md`.

---

_Authored by advisor session 2026-05-20 against phase-branch HEAD `6e16ee94f`. Absorbs `docs/research/brehon-rls-pmd-review.md` top-5 hardenings. The PRECON-1/2/3/4/5/6/7/8/9 decisions in §0.1 captured the user's 2026-05-20 directives; the planner does NOT re-derive them._
