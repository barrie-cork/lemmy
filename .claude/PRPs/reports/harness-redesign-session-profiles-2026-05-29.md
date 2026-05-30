# Harness redesign — session-profile resident core (`2026-05-29`)

**Branch:** `governance-v0`
**Scope:** structural redesign of the Claude Code ALWAYS-load corpus, keyed to session-type.
**Predecessor:** `.claude/PRPs/reports/harness-audit-2026-05-29.md` (conservative trim, applied `f9982bd6e`, ~1.8k won). This report answers the question that audit *explicitly deferred*: "what must be resident every session vs. JIT-loaded per session-type."
**Method:** exhaustive Pi + Claude-side heading-citation grep; section-level char measurement; live-token calibration. Read-only — no source file edited.

> **Token convention (load-bearing).** All estimates use **2.4 chars→token** for markdown, verified against the predecessor's `/context` actuals: advisor-orchestrator 47,410 chars / 19,800 live-tok = **2.39**; decision-queue (pre-trim) 30,596 / 12,300 = **2.49**; MEMORY.md 26,318 / 11,100 = **2.37**. The chars/4 estimate undercounts by **1.67×** and must not be used.

---

## 0. Verified baseline (current HEAD)

| File | Class | Chars | Live tok @2.4 |
|---|---|---:|---:|
| advisor-orchestrator.md | ALWAYS | 47,410 | 19,754 |
| decision-queue.md | ALWAYS | 29,181 | 12,158 |
| MEMORY.md (user) | ALWAYS | 26,318 | 10,965 |
| multi-lane-worktree.md | ALWAYS | 17,731 | 7,387 |
| branch-manager.md | ALWAYS | 11,958 | 4,982 |
| pmd-invariants.md | ALWAYS | 9,254 | 3,855 |
| CLAUDE.md (root) | ALWAYS | 7,467 | 3,111 |
| pmd-search-strategy.md | ALWAYS | 5,065 | 2,110 |
| phase-branch.md | ALWAYS | 3,934 | 1,639 |
| memory-injection.md | ALWAYS | 3,096 | 1,290 |
| post-task-retro.md | ALWAYS | 2,580 | 1,075 |
| circuit-breaker.md | ALWAYS | 2,056 | 856 |
| escalation.md | ALWAYS | 1,977 | 823 |
| integrator.md | ALWAYS | 1,703 | 709 |
| session-awareness.md | ALWAYS | 1,020 | 425 |
| gh-pr-fork-target.md | ALWAYS | 506 | 210 |
| no-destructive-defaults.md | ALWAYS | 385 | 160 |
| **TOTAL** | | **171,641** | **71,517** |

**This matches the brief's ~71k figure and the predecessor's `/context` actual (~70.8k).** The target is to cut from **71.5k → ≤22k** so total session baseline (≈28k fixed non-memory + memory) lands ≤50k.

`paths:`-scope mechanism is **confirmed live in this repo**: 5 SCOPED rule files carry valid `paths:` YAML frontmatter (e.g. `governance-log-entry-kind-registry.md` → `crates/.../governance/**`; `no-cargo-output-paste.md` → `crates/**` + `scripts/brehon/**` + `.github/workflows/**`). The 15 ALWAYS files have **no frontmatter** — verified, they open with `# Heading` directly. So scoping any of them is a one-edit frontmatter add, not new infrastructure.

---

## 1. Per-session-type minimal resident core

Three CC profiles run in this repo. The question for each: **what does a turn-1 reply (before invoking any skill) actually require resident?**

### 1a. Advisor session (this session) — governance-v0 or phase-lane, persistent

The decisive finding: **`/auto-phase` and `/auto-roadmap` skill bodies already contain the orchestration state machine as a self-sufficient compilation.** `~/.claude/commands/auto-phase.md` line 410 states verbatim: *"The state machine is a literal compilation of `.claude/rules/advisor-orchestrator.md` §'Stage-shape orchestration'."* Its Phase 2 routing table (lines 412-440) encodes every stage transition, the cohort-dispatch trigger, the §G4 cycle-count routing, and all six user gates. And `.claude/refs/auto-phase.md` (lazy-loaded by the skill, 26,041 chars) carries a duplicate **"Stage-shape orchestration (canonical contract)"** section (line 371) with the full Phase-1/Phase-2 detail.

Even more telling: the skill body **already instructs read-on-demand for the heaviest advisor-orchestrator sub-tables.** Line 388: *"The 'Mandatory file-class lesson injection' table in `.claude/rules/advisor-orchestrator.md` — only needed when authoring the next impl-task brief… **Read it inside the `impl-cohort-N` action handler, not at resume.**"* Line 158 cites `§5.3 cycle-count meta-rule` the same way. The skill is **designed** to pull those tables JIT.

**Consequence — what an advisor turn-1 actually needs resident:**

| Resident at turn 1 (advisor) | Needed only after a skill is invoked |
|---|---|
| advisor-orchestrator **§1 Polling loop** (the trigger surface — every conversational tick reads list_tasks / DQ) | **§4 Cohort dispatch** (only fires inside `/auto-phase` `impl-cohort-N`) |
| advisor-orchestrator **§3.1 Stage-shape** *index* + **§3.2 user gates** (the 6 gates are the never-skip contract; must be resident so a bare turn doesn't skip a gate) | **§5.1 Forbidden windows / §5.2 validate-pending-laptop / §5.3 §G4 classifier** (only fire on a validate-pending entry — i.e. mid-orchestration) |
| advisor-orchestrator **§5.6 Catch-fire procedures** (Pi-cited; the table of stop-conditions a bare triage turn must honour) | **§2.1–§2.5 Brief authoring + file-class lesson table** (only when authoring a Junior brief — always inside a stage transition) |
| advisor-orchestrator **§5.4 DQ triage decision tree** (a bare advisor turn answering a DQ needs the advisor-answer/catch-fire/user-relay split + commit-subject rule) | **§4.3 Cohort handover aggregation, §6 Subagent dispatch** (skill-driven) |
| decision-queue **Schema (v3) + Attribution integrity + Hard refusals + Who answers** (a bare turn writing a DQ answer needs schema + attribution) | decision-queue **ci-watcher mutation pattern, Polling-loop routing per kind** (consumed only when processing a validate-pending mutation — mid-orchestration; the ci-watcher itself reads it, not the advisor turn-1) |
| decision-queue **Mid-task visibility** (push discipline — applies to any DQ write) | decision-queue **Recipes 1–3** (already pointer-only to refs/dq-recipes.md) |
| multi-lane **Layout + Hard refusals + Session-start ritual** (CWD/lane safety — every session-start) | multi-lane **Lane modes (Mode A/B narrative) + Brief location/trunk→phase sync** (only when bm-cut'ing or authoring a brief — skill-adjacent) |
| pmd-invariants **all 5** (non-negotiable substrate invariants; cheap at 3.9k) | — |
| pmd-search-strategy (PMD is queried on most turns) | — |
| The 10 small floor files (circuit-breaker, escalation, integrator, session-awareness, gh-pr-fork-target, no-destructive-defaults, memory-injection, post-task-retro, phase-branch, CLAUDE.md, MEMORY.md) | — |

**The advisor's irreducible resident core is roughly: §1 + §3.1-index + §3.2 + §5.4 + §5.6 of advisor-orchestrator; the schema/attribution/mid-task half of decision-queue; the safety half of multi-lane; all of pmd-invariants + the floor files.** The *mechanism* halves (§4, §5.1-5.3 of advisor-orchestrator; ci-watcher-mutation + per-kind-routing of decision-queue; Lane-modes + brief-sync of multi-lane) are **skill-driven and already lazy-read by the skill body** — they are the JIT candidates.

### 1b. BM session — separate CC session, same worktree, runs `/bm-*`

`branch-manager.md` is its operating contract and must be resident (it is the BM's turn-1 substrate — file-ownership, autonomy table, refusals). The BM session does **not** need advisor-orchestrator §3/§4/§5 at all — those are advisor-only. It needs decision-queue **Attribution integrity + Mid-task visibility** (BM writes DQ entries) and phase-branch.md (it enforces it). It does **not** need multi-lane Lane-modes, pmd-invariants beyond the canonical-path guard, or the orchestration machinery.

**BM resident core:** branch-manager.md (all), decision-queue Attribution+Mid-task+Schema, phase-branch.md, gh-pr-fork-target.md, no-destructive-defaults.md, the floor files. **It pays ~22k today for advisor-orchestrator + multi-lane Lane-modes + decision-queue mechanism that it never uses.**

### 1c. Impl session — rare on laptop, mostly Junior on daemon, runs `/prp-implement`, writes `crates/**`

An impl session needs the **cargo/Rust discipline** (which is already SCOPED to `crates/**`: cargo-output-capture, no-cargo-output-paste, view-crate-selectable-template, governance-log-entry-kind-registry — these load *correctly* the moment it Reads a `crates/**` file). It needs decision-queue **Schema + blocker-kind + Mid-task + Attribution** (impl raises blockers/validate-pending). It does **not** need advisor-orchestrator §2-§6, branch-manager.md, or multi-lane Lane-modes.

**Impl resident core:** decision-queue (schema/blocker/mid-task/attribution), phase-branch.md, the floor files. The Rust-specific rules are *already* scoped correctly. **An impl session pays ~30k today (advisor-orchestrator + branch-manager + multi-lane + decision-queue mechanism) for content it never reads.**

**Cross-cutting insight:** the three profiles share only ~12-15k of genuinely-universal content (decision-queue schema/attribution/mid-task, pmd-invariants, the floor files, phase-branch, gh-pr-fork-target, no-destructive). Everything above that is **profile-specific** and currently force-loaded on all three because the files have no `paths:` frontmatter and no skill-body home.

---

## 2. Pi-citation freeze map (the gating constraint)

A heading with **≥1 citation by name** (Pi OR a live Claude-side skill/command) is **FROZEN** — it must remain a heading in its current file, or the citing prose silently dangles. A heading with **0 citations** is **relocatable** (the *prose under it* can move; if the heading itself has no citers, the heading can move too).

**Grep commands run:**
```bash
# Pi side (the hard constraint):
grep -rn 'advisor-orchestrator' .pi          # 13 hits across brehon-clarify, brehon-verify, impl-task
grep -rn 'decision-queue'       .pi          # ~40 hits; ci-watcher SKILL cites mutation contract ×11
grep -rn 'multi-lane-worktree'  .pi          # 0 hits
grep -rn 'branch-manager|pmd-invariants' .pi # branch-manager cited as FILE ×many; pmd-invariants 0
grep -n  'When to override|Forbidden execution|Catch-fire|Stage-shape|G4 classifier' .pi
sed -n '12p' .pi/skills/ci-watcher/SKILL.md   # the omitted long line — cites §G4 + mutation
# Claude side (also freezes a heading):
grep -rn '<file>' .claude/skills .claude/commands .claude/agents .claude/refs ~/.claude/commands ~/.claude/skills \
  | grep -v retro-harvest-workspace | grep -iv 'reports/'   # exclude test fixtures + report prose
```
Note: `AGENTS.md` rules #1/#2 tell Pi sessions **not to read** `CLAUDE.md` / `.claude/rules/*.md` directly. The Pi citations are *prose pointers inside `.pi/` prompts/skills* that send a Pi subagent to a **named section** of a Claude rule file when a task touches it. So the freeze is one level removed but real: the cited heading must resolve in the file Pi is told to Read.

### advisor-orchestrator.md

| Heading | Pi cites | Claude-side cites | Verdict |
|---|---:|---:|---|
| § (title) Advisor as orchestrator | 0 | 0 | relocatable (but it's the H1) |
| §1 Polling loop | 0 | 1 (`§1 "Polling loop"`, `§1 "Pre-compact handover discipline"`) | **FROZEN** (Claude-side) |
| §2 Brief authoring | 0 | 1 (`§2.4` table cited by auto-phase line 388) | **FROZEN** (the §2.4 table heading) |
| §2.1 Junior task template | 0 | 0 | relocatable |
| §2.2 Brief shape | 0 | 0 | relocatable |
| §2.3 Pre-queue lesson check | 0 | 0 | relocatable |
| §2.4 Mandatory file-class lesson injection | 0 | **1 (auto-phase body Reads it on demand)** | **FROZEN** (heading is the read-target) |
| §2.5 Plan §5 complexity-score | 0 | 0 | relocatable |
| §3 Stages and gates | 0 | 1 (`§3 "Stages and gates"`) | **FROZEN** |
| §3.1 Stage-shape orchestration | **3** (brehon-clarify ×2, brehon-verify ×1) | 4 | **FROZEN** (heaviest Pi anchor) |
| §3.2 Mandatory user gates | 0 | 0 | relocatable (but load-bearing — keep resident) |
| §3.3 Clarify gate | 0 | 2 (`§3.3`) | **FROZEN** |
| §3.4 DoD smoke test | 0 | 2 (`§3.4`) | **FROZEN** |
| §3.5 Watchpoint specificity | 0 | 0 | relocatable |
| §3.6 Canonical-schema-first gate | 0 | 0 | relocatable |
| §3.7/3.8 Dogfood + schema-retrofit | 0 | 0 | relocatable (already pointer to refs) |
| §3.9 Verify gate | 0 | 0 | relocatable |
| §3.9.1 Conformance-audit detection | 0 | 0 | relocatable |
| §4 Cohort dispatch | 0 | 1 (`§4`, `§"Cohort dispatch sequence"`) | **FROZEN** (heading) — body relocatable |
| §4.1 Cohort dispatch sequence | 0 | 1 | **FROZEN** (heading) — sub-prose relocatable |
| §4.2 Cohort dispatch refusals | 0 | 0 | relocatable |
| §4.3 Cohort handover aggregation | 0 | 0 | relocatable |
| §4.4 Notes | 0 | 0 | relocatable |
| §5 Validation, classification, recovery | 0 | 0 | relocatable (H2) |
| §5.1 Forbidden execution windows | **1** (impl-task SKILL) | 3 | **FROZEN** (Pi) — body already in refs |
| §5.2 validate-pending-laptop handler | 0 | 2 (`§5.2`) | **FROZEN** — body already in refs |
| §5.3 §G4 classifier | **3** (ci-watcher SKILL, "§G4 classifier") | 3 (`§5.3`) | **FROZEN** (heading) — allowlist-table relocatable IF re-anchored |
| §5.4 DQ triage decision tree | 0 | 1 (`"DQ triage decision tree"`) | **FROZEN** |
| §5.5 Retro-bypass observability | 0 | 0 | relocatable |
| §5.6 Catch-fire procedures | **3** (brehon-verify) | 2 | **FROZEN** (Pi) |
| §6 Subagent delegation | 0 | 1 (`§6 "Subagent delegation"`) | **FROZEN** (heading) — sub-prose relocatable |
| "When to override" | **1** (impl-task SKILL line 42) | 1 | **DANGLING TODAY** — cited but NOT a heading (lives inline in §5.1). Pre-existing defect; out of this audit's scope but flag it. |

### decision-queue.md

| Heading | Pi cites | Claude-side cites | Verdict |
|---|---:|---:|---|
| §Schema | 0 | 0 | relocatable (but tiny + load-bearing) |
| §Schema (v3) | 0 | 2 (`§"Schema (v3)"`) | **FROZEN** |
| §Archive policy | 0 | 0 | relocatable (already pointer to refs/dq-mechanics) |
| §Next-id calculation (v3 — canonical) | 0 | 0 | relocatable |
| §Next-id calculation (pre-v3 — historical) | 0 | 0 | **relocatable** (already has refs pointer; pure history) |
| §What does NOT get archived | 0 | 0 | relocatable (already pointer) |
| §When to use / §How to write a question | 0 | 0 | relocatable |
| §kind: blocker/log/clarify/validate-pending | **2** (ci-watcher, impl-task `"kind:"`) | 1 | **FROZEN** |
| §ci-watcher mutation pattern (option 2) | **11** (ci-watcher SKILL) | 1 | **FROZEN — hardest anchor in the corpus** |
| §Deprecated kinds + two-phase Shape-G | 0 | 0 | **relocatable** (already has refs pointer; historical) |
| §Polling-loop routing per kind | 0 | 0 | relocatable (advisor-only mechanism) |
| §Recipes (copy-pasteable) | 0 | 1 (Recipes 1–3 noted as bodies-already-in-refs) | **relocatable** (bodies already in refs/dq-recipes) |
| §Hard refusals (write-side) | 0 | 0 | relocatable (but load-bearing — keep) |
| §After writing a question / §How to read an answer / §Who answers | 0 | 0 | relocatable |
| §Attribution integrity (load-bearing) | **1** (bm-pi, ci-watcher) | 5 (`§"Attribution integrity"`, `"§Detection"`) | **FROZEN — heavily cited** |
| §Concurrency | 0 | 0 | relocatable |
| §Mid-task visibility (Junior worktrees) | **2** (bm-task, impl-task) | 1 | **FROZEN** |
| §Subagents and attribution | 0 | 0 | relocatable (overlaps Attribution integrity) |

### multi-lane-worktree.md

| Heading | Pi cites | Claude-side cites | Verdict |
|---|---:|---:|---|
| §Why this rule exists | 0 | 0 | relocatable |
| §Layout | 0 | 0 | relocatable (but load-bearing safety — keep resident) |
| §Lane modes / Mode A / Mode B | **0** | 0 | **relocatable** |
| §How to tell which mode you're in | 0 | 0 | relocatable |
| §Brief location and trunk→phase sync | **0** | 0 | **relocatable** (SSH recipe — pure mechanism) |
| §Lifecycle | 0 | 3 (`§"Lifecycle"`) | **FROZEN** (Claude-side) |
| §Session-start ritual | 0 | 1 (`§"Session-start ritual"`) | **FROZEN** |
| §Hard refusals | 0 | 0 | relocatable (but load-bearing — keep) |
| §Worktree-aware DQ id discipline | 0 | 0 | relocatable (already pointer to refs) |
| §PMD is cross-lane shared | 0 | 0 | relocatable (overlaps pmd-invariants #1) |
| §Daemon side / Migration plan | 0 | 0 | relocatable (already pointer) |

**`multi-lane-worktree.md` has ZERO Pi citations.** Only 2 headings are frozen by Claude-side skills (`Lifecycle`, `Session-start ritual`). This is the **least-constrained large file** — the entire Lane-modes + brief-sync + daemon narrative (≈13k of its 17.7k chars) is relocatable from a freeze standpoint.

### branch-manager.md

| Heading | Pi cites | Claude-side cites | Verdict |
|---|---:|---:|---|
| (file as a whole) | **many** (`.pi/agents/bm-pi.md`, `.pi/skills/bm-task`, `.pi/skills/branch-manager`, all 9 bm-* prompts) | many | The FILE is Pi-mandatory-OUT (per predecessor + `.pi/skills/branch-manager/SKILL.md` line 12: *"already loaded into your context through Claude Code's project-rules inheritance"*) |
| §File ownership boundaries (HARD) | 0 | 1 (`"File ownership boundaries"`) | **FROZEN** |
| §Hard refusals (`## Hard refusals` referenced by bm-pi.md line 39) | implicit | — | **FROZEN** |
| All other headings | 0 | 0 (cited by section *content*, not anchor) | relocatable in principle, but see below |

**branch-manager.md is special:** `.pi/skills/branch-manager/SKILL.md` line 12 explicitly relies on the file being **auto-loaded into the BM subagent's context via project-rules inheritance** (*"Do not re-read it as a first step… rely on what the harness already provided"*). This is the one file where the *always-load behaviour itself* is a Pi-relied-upon contract — Pi's BM agent assumes it is resident, not that it must Read it. **Scoping it would break Pi's BM agent assumption.** It stays always-load. (But it's only ~5k tok, and it's profile-specific: the *advisor* and *impl* sessions pay for it needlessly — see §4.)

### pmd-invariants.md

**0 Pi citations, 0 Claude-side heading citations.** Fully relocatable from a freeze standpoint. But it's the substrate-invariants file (canonical PMD path, two-systems, write-time-embedding, LESSON-trailer, SessionStart guard), cheap at 3.9k tok, and genuinely cross-profile. **Keep resident** — relocating it buys little and risks a substrate footgun.

---

## 3. Relocation plan with token deltas

Two distinct levers. **Lever A** (Pi-safe extraction within a file, heading stays) is what the predecessor already exhausted (~1.8k). **Lever B** (the structural lever this report introduces): move profile-specific mechanism into the skill body / refs the skill already loads, OR `paths:`-scope a profile-specific file. Lever B is where the real reduction lives.

### Lever B1 — Relocate the orchestration-mechanism halves into `/auto-phase` + `/auto-roadmap` refs

These sections fire **only during active orchestration**, the skill body already Reads them on demand (auto-phase line 388, 158), and the canonical contract is already duplicated in `refs/auto-phase.md`. Keep each FROZEN heading + a one-line "mechanism lives in refs/auto-phase.md §X; read JIT" pointer; move the body.

| Chunk (advisor-orchestrator) | Frozen heading kept? | Chars moved | ~tok removed | Stops paying |
|---|---|---:|---:|---|
| §4.1 sequence body (steps 1–9) + §4.2 refusals + §4.3 aggregation + §4.4 | §4 + §4.1 headings stay as stubs | ~6,800 of 7,354 | **~2,830** | all 3 profiles (advisor reads via skill) |
| §5.3 §G4 allowlist + non-allowlist tables + callsite/pre-push discipline + anti-paraphrase gate | §5.3 heading stays; **re-anchor the verbatim-blockquote source pointer** (brief template line cites "§G4 classifier table" — must repoint to refs) | ~6,000 of 16,535 (§5 whole) | **~2,500** | all 3 (advisor reads via skill on a validate-fail) |
| §2.1/2.2/2.3/2.5 brief-authoring prose (keep §2.4 table — frozen read-target — resident OR move table to refs and repoint auto-phase line 388) | §2 + §2.4 headings stay | ~4,500 of 8,604 | **~1,875** | advisor (read when authoring a brief) |
| §3.5/§3.6/§3.7-8/§3.9/§3.9.1 gate *detail* (keep §3.1 index + §3.2 gates + §3.3/§3.4 resident) | §3 + frozen sub-headings stay | ~2,500 | **~1,040** | advisor (read at the relevant gate) |
| §6 subagent-dispatch sub-prose (§6.1/6.2 already in refs; §6.3 stays) | §6 heading stays | ~1,200 of 2,346 | **~500** | advisor |
| §1 incident-narrative residue (predecessor already moved the big ones) | §1 heading stays | ~600 | **~250** | all 3 |
| **advisor-orchestrator subtotal** | | **~21,600** | **~9,000** | |

advisor-orchestrator drops from **19,754 → ~10,750 tok** if all of B1 lands. The resident remainder = §1 statement + §3.1-index + §3.2 + §3.3/§3.4 + §5.1/§5.2 (already thin) + §5.4 + §5.6 + frozen heading stubs.

### Lever B2 — Relocate decision-queue mechanism + history into refs/dq-mechanics (exists) + refs/dq-recipes (exists)

| Chunk (decision-queue) | Frozen heading kept? | Chars moved | ~tok removed | Stops paying |
|---|---|---:|---:|---|
| §Polling-loop routing per kind (advisor-only mechanism) | heading stays as stub | ~2,400 | **~1,000** | BM + impl (advisor reads via skill) |
| §Next-id pre-v3 (history) + §Deprecated kinds + two-phase Shape-G (history) | headings stay as stubs | ~2,200 | **~915** | all 3 |
| §Recipes 1–3 (bodies already in refs/dq-recipes) | heading stays | ~1,800 | **~750** | all 3 |
| §Concurrency + §Subagents-and-attribution (overlaps §Attribution integrity) | §Subagents heading is FROZEN-adjacent — verify; §Concurrency relocatable | ~2,000 | **~830** | all 3 |
| §Archive policy detail + §What-does-NOT-get-archived (already pointer) | headings stay | ~1,000 | **~415** | all 3 |
| **decision-queue subtotal** | | **~9,400** | **~3,900** | |

decision-queue drops from **12,158 → ~8,250 tok**. The resident remainder = Schema + Schema-v3 + kind-definitions + ci-watcher-mutation (the 11× Pi anchor — **stays**) + Attribution integrity + Mid-task visibility + Hard refusals + Who-answers. **The ci-watcher mutation pattern (11 Pi citations) is the single hardest constraint and cannot move — but it's only ~1.8k chars.**

### Lever B3 — `paths:`-scope `multi-lane-worktree.md` Lane-modes, OR relocate to refs

multi-lane has **0 Pi citations**; only `Lifecycle` + `Session-start ritual` are frozen (Claude-side). Two options:

- **B3a (relocate):** move Lane-modes + Mode-A/B + Brief-location/trunk→phase-sync + Daemon + Migration narrative to `refs/multi-lane-mechanics.md` (exists). Keep `Layout` + `Hard refusals` + `Session-start ritual` + `Lifecycle` (frozen) resident. ~10,000 chars moved → **~4,170 tok removed**, all 3 profiles stop paying. multi-lane drops 7,387 → ~3,200 tok.
- **B3b (scope):** the whole file is only relevant when ≥2 lanes are active. But there is **no file-glob that signals "multi-lane is active"** — it's a git-worktree-count condition, not a path Read. So `paths:`-scope does NOT fit multi-lane. **B3a is the only viable lever here.**

### Lever B4 — Profile-scope branch-manager.md? **NO.**

`.pi/skills/branch-manager/SKILL.md` line 12 relies on it being **resident via inheritance**. Scoping it breaks Pi's BM agent. It stays always-load (~5k tok). This is an irreducible cost the advisor and impl profiles pay for a file they don't use — accepted as a dual-harness tax.

### Total achievable always-load reduction (Lever B, honest)

| Lever | ~tok removed |
|---|---:|
| B1 advisor-orchestrator → auto-phase refs | ~9,000 |
| B2 decision-queue → dq-mechanics/dq-recipes | ~3,900 |
| B3a multi-lane → multi-lane-mechanics | ~4,170 |
| **Lever B total** | **~17,070** |
| + Lever A residue (predecessor-style, already mostly done) | ~500 |
| **Grand total** | **~17,570** |

**Post-redesign always-load ≈ 71,517 − 17,570 = ~53,950 tok.** Plus the independent MEMORY.md byte-prune lever (it's 10,965 tok = 15% of the load; a structural rewrite of its index — moving CLOSED-lane detail to topic files — could shave ~3-4k more, landing ~50k).

**≤22k is NOT reachable.** See §5.

---

## 4. The decision the user faces

**The structural tradeoff is real and quantified.** Lever B1+B2 move the orchestration *mechanism* (cohort-dispatch steps, §G4 allowlist, per-kind DQ routing) out of always-load and into the `/auto-phase` skill body + refs that the skill already loads. **This is free WHEN `/auto-phase` or `/auto-roadmap` is driving** — the skill Reads the refs on demand, exactly as its body already instructs (auto-phase line 388).

**The risk surfaces on a bare advisor turn that does NOT invoke a skill.** Concretely:

- An advisor doing **ad-hoc triage** (user asks "should I dispatch this as a cohort?" or "this validate-pending failed — auto-fix or catch-fire?") in a plain conversational turn would **no longer have §4 cohort-dispatch discipline or the §5.3 §G4 allowlist resident.** It would need to either (a) invoke `/auto-phase` (which re-loads the mechanism), or (b) `Read .claude/refs/auto-phase.md` / `Read` the relocated section before answering.
- **What's gained:** ~13k tokens off every session that *isn't* actively orchestrating — i.e. every BM session, every impl session, and every advisor conversational/planning/triage turn that hasn't started a phase.
- **What's risked:** a bare advisor turn answering a cohort/§G4 question *from memory* could mis-apply the discipline (e.g. forget the YAML-overlap degrade-to-serial rule, or auto-fix a non-allowlist failure) because the contract isn't resident. Today that contract is always in front of it.

**Mitigations that make the tradeoff acceptable:**
1. Keep the **§4 / §5.3 / per-kind-routing *headings* + a one-line "read refs/auto-phase.md §X before acting on a cohort/validate-fail decision outside /auto-phase" stub** resident. The stub (≈40 tok each) preserves the *signal* that discipline exists and where to load it — the agent knows to Read before answering, even on a bare turn.
2. The six **user gates (§3.2) + catch-fire procedures (§5.6) + DQ triage decision tree (§5.4) stay resident.** These are the "never skip" safety contract; they are NOT moved. The moved content is *execution mechanism*, not *gate/safety policy*.
3. The advisor's de-facto pattern is already skill-driven (`/auto-phase`, `/auto-roadmap`, `/check-dq`, `/brehon-clarify`, `/brehon-verify` cover virtually every orchestration action). A truly-bare orchestration turn is rare; when it happens, the stub-pointer routes the Read.

**The honest framing for the user:** *"Moving cohort-dispatch + §G4 + per-kind-DQ-routing into the skill bodies saves ~13k tokens on every non-orchestrating session, at the cost that an advisor doing ad-hoc orchestration triage (not via /auto-phase) must Read a refs file or invoke the skill before applying that discipline — the gate/safety/attribution policy stays resident either way."*

---

## 5. Recommended target — the defensible floor for THIS dual-harness repo

**≤22k is not achievable without unacceptable Pi-breakage or discipline-loss. The defensible floor is ~50k.** Justification:

### What is irreducibly resident (the floor), with the Pi-freeze map applied

| Resident component | ~tok | Why it cannot move |
|---|---:|---|
| decision-queue: Schema + Schema-v3 + kind-defs + **ci-watcher mutation (11× Pi)** + Attribution (6× cited) + Mid-task (Pi) + Hard refusals + Who-answers | ~8,250 | The DQ contract is the cross-harness substrate; ci-watcher-mutation has 11 Pi citations |
| advisor-orchestrator: §1 + §3.1-index + §3.2 gates + §3.3/§3.4 (Pi+Claude cited) + §5.1/§5.2 (Pi) + §5.4 + §5.6 (Pi) + frozen stubs | ~10,750 | §3.1 (3 Pi), §5.6 (3 Pi), §3.3/§3.4 (Claude); gates+catch-fire are safety policy |
| branch-manager.md (whole — Pi-resident-by-inheritance) | ~4,982 | `.pi/skills/branch-manager` relies on it being always-loaded |
| multi-lane: Layout + Hard refusals + Session-start ritual + Lifecycle (Claude-cited) | ~3,200 | session-start safety + 2 frozen headings |
| pmd-invariants (all 5) | ~3,855 | substrate invariants; cheap; cross-profile |
| pmd-search-strategy | ~2,110 | PMD queried most turns |
| CLAUDE.md | ~3,111 | the root project contract (four-role model, hard constraints, paths) |
| phase-branch + memory-injection + post-task-retro + circuit-breaker + escalation + integrator + session-awareness + gh-pr-fork-target + no-destructive | ~7,000 | floor discipline files; below extraction threshold; broadly cited |
| MEMORY.md (post structural index-rewrite) | ~7,000–11,000 | the live-state index; can shrink via topic-file offload but not vanish |
| **Floor total** | **~52,000–56,000** | |

With aggressive MEMORY.md index restructuring (offload all CLOSED-lane detail to topic files, keep only one-line active-lane pointers) the floor reaches **~50k**.

### Why ≤22k is structurally impossible here

1. **The Pi-freeze map alone pins ~8k (decision-queue) + the branch-manager file (~5k) + the Pi-cited advisor-orchestrator headings.** The 11× ci-watcher-mutation citation, the 6× Attribution citation, the 3× Stage-shape / 3× Catch-fire citations, and `.pi/skills/branch-manager`'s resident-by-inheritance assumption are hard contracts. Breaking any silently breaks Pi.
2. **decision-queue's schema/attribution/mid-task IS the dual-harness contract** — Junior subagents (impl, bm, ci-watcher) AND Pi subagents all write/read DQ against it. It cannot be JIT'd to a skill because the *subagents that consume it are not the advisor and do not invoke advisor skills.* The predecessor and `harness-audit/helpers/report-template.md` line 68 both correctly mark it mandatory-OUT.
3. **There is no `paths:` glob for "orchestration is active" or "multi-lane is active"** — these are session-role and worktree-count conditions, not file-Read events. So the cleanest JIT primitive (`paths:`-scope) does **not** apply to the advisor-orchestrator or multi-lane content. The only JIT route is skill-body relocation (Lever B), which works only for content the skill loads — i.e. mechanism, not the always-resident gate/safety/schema policy.
4. **22k would require dropping the safety policy from always-load** (gates, catch-fire, attribution, DQ schema) — which is exactly the discipline the four-role model depends on and exactly what the predecessor's "discipline is the lever, not cutting" conclusion warned against.

### Recommended action

1. **Apply Lever B1 + B2 + B3a (~17k saved → ~54k).** Relocate orchestration mechanism (cohort §4 body, §G4 §5.3 tables, brief-authoring §2.x prose, gate-detail §3.5-3.9) to `refs/auto-phase.md`; DQ mechanism+history to `refs/dq-mechanics`/`refs/dq-recipes`; multi-lane Lane-modes+brief-sync+daemon to `refs/multi-lane-mechanics`. **Keep every FROZEN heading + a one-line read-on-demand stub.** Re-anchor the brief-template's "§G4 classifier table" verbatim-source pointer to the new refs location (otherwise the anti-paraphrase gate dangles).
2. **Restructure MEMORY.md** (separate `memory-prune` structural pass): move all CLOSED-lane detail to topic files, keep one-line active pointers → ~3-4k more → **~50k floor.**
3. **Adopt a `verify-rule-anchors.sh` guard** (flagged absent in retro-harvest eval O11) that fails CI if any `.pi/` or Claude-side prose cites a heading that no longer exists in the target file. This makes the freeze map *enforced*, not just documented — the single highest-leverage protection against silently breaking Pi during any future relocation.
4. **State the §4 explicitly to the user before applying B1/B2:** the ~13k saved on non-orchestrating sessions costs a refs-Read (or skill-invoke) for ad-hoc orchestration triage on a bare advisor turn. The gate/safety/attribution policy stays resident regardless.

**Bottom line:** The session-profile insight is correct and yields a real ~17k structural win that the conservative trim could not — but it lands at **~54k, ~50k with the MEMORY.md rewrite**, not 22k. The ≤22k target collides with the Pi-citation freeze map (especially the 11× ci-watcher-mutation contract and the resident-by-inheritance branch-manager assumption) and the irreducible cross-harness DQ-schema + gate/safety policy. **~50k is the defensible floor for this dual-harness repo; pursuing 22k would require breaking Pi or dropping the four-role safety discipline.** This corroborates and structurally extends the predecessor's "growth-discipline is the lever" conclusion: the one-time structural win is ~17k (10× the conservative trim's 1.8k), but it does not reach an arbitrary 22k target, and the durable lever remains authoring new mechanism in refs/skill-bodies from the start.
