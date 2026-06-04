# v1 Close-Out — Cleanup & Refactor Plan (hybrid /workflows + four-role)

> **Status:** APPROVED 2026-06-04. Execution handed to the user in a dedicated `brehon-fork-closeout` worktree session. Author: advisor session (canonical `brehon-fork`, `governance-v0`).
> **Plan-file home:** `C:\Users\barri\.claude\plans\steady-seeking-lampson.md` (this file) + tracked copy at `.claude/PRPs/plans/v1-closeout.plan.md` on `governance-v0`.
>
> **Approval decisions (2026-06-04):**
> 1. **Phase 4 / deps-r2** — RUN NOW, gate only T2. T1 (webmention inline), T3 (tar dispute), T4 (wasmtime log) proceed immediately in the close-out lane; T2's `[patch.crates-io]` Cargo.toml edit waits for M1 merge.
> 2. **Phase 5 / wasmtime** — KEEP DEFERRED + document x86_64-not-vulnerable rationale + suppress all 13 in Dependabot + **set a `/schedule` watch on `extism/extism` PR #847** to adopt the upstream release the moment it lands.
> 3. **Promotion** — promote to tracked `.claude/PRPs/plans/v1-closeout.plan.md` (done at Phase 0).
> 4. **First workflow** — Phase 3 carry-patch audit is the small cost-gauge run before broader sweeps.
>
> **Execution owner:** the USER, in a fully-harnessed `brehon-fork-closeout` session (PMD + MCP + hooks wired per Phase 0). This advisor session only prepares the handoff.

---

## Session progress & state  (LIVING — newest entry on top)

> Append a dated block each working session. Keep it factual: what shipped, what's gated, what the next concrete action is. This is the resume surface — a fresh session reads this section + the bootstrap handover and knows where it stands.

### State snapshot (as of 2026-06-04, session `d3e7ac99`)

| Axis | Value |
|---|---|
| Lane / branch | `brehon-fork-closeout` / `phase-v1-closeout` (HEAD `1c7084622` at session open) |
| Mode | Mode A (dedicated lane worktree) — own CC session, laptop-local |
| Harness | **GREEN** — Probes 1+2 ✅ exit 0; Probe 3 (e2e `--no-run`) compiling final workspace tier (`lemmy_apub_send` → `lemmy_server` next); Probe 4 (negative exit-code test) chained after. Flag NOT yet set. |
| M1 gate | **CLOSED** — `phase-m1-b` is **+22 commits** over `governance-v0` (tip `dc5b93ba8`, M1 mid-Task-5, advanced this session). Phases 6–8 GATED; NO-ELITEDESK live. |
| Phases done | Phase 0 (bootstrap) ✅ |
| Next action | Finish harness sign-off (await Probe 3/4 → verify Probe 4 returns **non-zero** → `touch .claude/audit-phase-v1-closeout-complete.flag`). Then user picks first phase — recommended **Phase 3 carry-patch audit** (the cost-gauge 🟦 run, read-only, daemon-free). |

**Executable NOW** (laptop-local, no daemon, M1-isolation-safe): Phase 3 audit (cost-gauge first 🟦), Phase 1b/1c/1d hygiene, Phase 2 doc-drift, the **DQ archive** (545 KB / 273 resolved — over BOTH triggers), Phase 4-**T3** (tar-dispute, GitHub UI) + 4-**T4** (wasmtime-log DQ), Phase 5 **decision** (recommend keep-deferred + `/schedule` watch on extism PR #847).
**BLOCKED until M1 merges:** Phase 4-**T1** (webmention inline, `api_utils/src/utils.rs` — was 🟩 four-role, now needs user decision: defer vs laptop-local impl), 4-**T2** (`[patch.crates-io]` Cargo.toml — M1 overlap), **Phases 6–8** (e2e split / type-state / TODO-sweep — M1 owns those files).

### 2026-06-04 — session `d3e7ac99` (harness verification + lane-setup audit)

- **Harness verified GREEN before any plan work** (per `pre-phase-harness-audit.md` + user instruction "first ensure the harness system is fully working"). Bootstrap artifacts (submodules, `.mcp.json`, `.env`, `settings.local.json`), PMD HTTP daemon (hybrid search round-trips), Junior daemon healthy, all SessionStart hooks wired. Cargo probes 1+2 pass; 3+4 in flight at session close.
- **GAP 1 fixed — Telegram completion hook was broken.** `list_hooks` showed hook ID 1 `active` but with `LAST ERROR: BuildMessage: ModuleNotFound`, never triggering (daemon restart May 29 left a stale check_fn). Removed ID 1 → recreated as **ID 2** (`junior-task-terminal-notify`), clean. *(Read-only-equivalent daemon op — no task dispatch; honors NO-ELITEDESK.)*
- **GAP 2 fixed — missing lesson reconstructed.** `feedback_daemon_telegram_completion_hook.md` was cited by always-load `advisor-orchestrator.md` §1 + 4 handovers but absent on disk. Authored it (3-state decision table, recreate-via-natural-language recipe, hook-ID-not-stable note, junior.db-vs-MCP-store topology caveat).
- **NO-ELITEDESK directive captured** (user, 2026-06-04). Recorded in PMD (`project_closeout_no_elitedesk_m1_active.md`) + MEMORY.md HARD RULE + folded into Phase 4 / cross-cutting guardrails of this plan (commit `1c7084622`).
- **Lane-setup report authored** — `.claude/PRPs/reports/v1-closeout-lane-setup-report.md` (forward-looking learnings for future isolated-lane bootstraps; TL;DR table, the 2 gaps, daemon-topology finding, cargo-probe table, hygiene baseline, improvement checklist).
- **Hygiene debt quantified** for Phase 1: DQ **545 KB / 273 resolved** (over both archive triggers), **55 active plan files**, 8 worktree dirs.
- **Plan validated self-consistent** with the live NO-ELITEDESK directive (Phase 4 lines 151–159 + guardrail line 241 already encode it). Phase 4 is *partially* executable: T3/T4 are laptop-local/daemon-free; only T1/T2 are gated.
- **Deferred (not done):** MEMORY.md is at 203 lines (+1 from the HARD-RULE add) — a proper prune is **Phase 1c's job** (milestone-prune skill, user-gated), not an ad-hoc trim. Flagged as hygiene item in the setup report.

---

## Context — why this plan exists

v1 has shipped. Every lane in `.claude/PRPs/v1-roadmap.json` is `done` — JM, SL, AD, RT (r1–r5), SR (ship-1→3), federation-inbound (a–e), redaction-r1, quality (r1→r3c), plus the meta-lanes (validate-agent, rls-r1, dq-schema-r1). All 11 v0 endpoints are implemented and e2e-tested. The codebase is now a mature Lemmy fork (`governance-v0`) carrying a 91-file governance layer woven across 8 crate sub-trees.

Shipping at this pace left three classes of accumulated debt that a deliberate close-out should retire **before** the platform takes on the next milestone's weight:

1. **Code-side debt** — an 18,526-line `crates/server/tests/e2e.rs` monolith (a repeated Junior Edit-hang hazard), 6 `TODO(type-state)` retrofit sites in `governance/`, 38 governance `TODO/FIXME` markers.
2. **Repo & meta hygiene** — 6 orphaned worktree directories on disk, 54 active plan files (most for shipped sub-phases), 217 reports, a 194-lesson corpus, and roadmap/registry documentation drift.
3. **Deferred security/deps** — `v1-deps-r2` (6 Dependabot alerts) and `v1-deps-r3` (13 wasmtime alerts via extism), both explicitly parked "post-v1-ship."
4. **Carry-patch upstreaming** — 11 `TODO(brehon-fork)` markers; 4 are generic, upstreamable Lemmy fixes that could be contributed and then deleted from the tree.

**The hard constraint:** the next milestone, **M1 (chat infrastructure / "v2")**, is *in flight right now* in the `brehon-fork-validate` worktree (`phase-m1-b`, Tasks 0–7). M1 implements/edits `crates/server/tests/e2e.rs`, `crates/api/api/src/governance/{mod,messaging_config,admin_config}.rs`, and `crates/db_schema/{newtypes,…/governance/…,schema}`. **This overlaps the e2e-split and type-state-retrofit surfaces directly.** Per the user's decision, this plan **isolates in its own worktree and defers every `crates/` edit until M1 merges.** Meta/hygiene and deps/upstreaming *analysis* run now; `crates/` *refactoring* is staged behind the M1 landing.

**Intended outcome:** a clean v1 baseline — empty stale-worktree set, archived shipped plans, reconciled docs, a triaged+filed upstreaming set, a green security posture (or documented suppressions), and (post-M1) a decomposed e2e suite + type-state-hardened governance handlers — without ever racing the live M1 lane.

---

## Execution model — the hybrid (this is the core design)

Per the user: *not all tasks suit `/workflows`; the plan is hybrid.* The Workflow tool runs a background JS script that fans out subagents — its sweet spot is **parallel read/audit + adversarial verification** where intermediate results stay out of the main context. Its hard limits (confirmed against the Claude Code guide): the **script itself has no file/shell access** (only the subagents it spawns do), and a workflow **cannot pause for mid-run user input** — only permission prompts. So anything needing a **user gate mid-flight** (CR triage, merge confirm, ADR-risk calls) must live *outside* a workflow.

Every phase below is tagged with one **execution lane**:

| Lane | When it applies | Mechanism |
|---|---|---|
| 🟦 **workflow** | Parallel fan-out over many files/sites; audit + adversarial verify; read-heavy classification. No mid-run user gate. | `Workflow` tool (a JS script per phase). User opt-in already given ("optimise it for /workflows"); launch still shows the per-run approval prompt. |
| 🟩 **four-role** | `crates/` code changes that need plan→approval→Junior-impl→CR→merge gates. | Brehon `/auto-phase` pipeline (planning/impl-task/bm-task/ci-watcher) — the existing machinery. |
| 🟨 **manual/advisor** | Single careful sequential actions, inspect-then-confirm deletes, ADR-risk decisions, or launching/gating the other two lanes. | Advisor session directly (this session or a successor), with explicit user confirmation where flagged. |

**Why hybrid beats all-workflow:** the audits (which files, which sites, what state) are embarrassingly parallel and benefit from a workflow's fan-out + verify; the *mutations* of shipped governance code are exactly the high-stakes, gated, one-careful-change-at-a-time work the four-role pipeline was built for — and the destructive filesystem ops need a human's eyes before they run.

**Workflow cost gate (applies to every 🟦 phase):** the first workflow run in this close-out should be scoped small (e.g. the 11-file carry-patch audit, not a whole-repo sweep) to gauge token burn before the broader sweeps. Caps: 16 concurrent / 1000 total agents per run; the user can stop any run without losing completed work and approves each launch.

---

## Phase map (by conflict-risk, not by topic)

```
NOW (zero M1 conflict — no crates/ mutation)
├─ Phase 0  Bootstrap the isolated close-out worktree          🟨 manual/advisor
├─ Phase 1  Repo & meta hygiene                                🟦 workflow (audit) → 🟨 (apply)
├─ Phase 2  Documentation-drift reconcile                      🟦 workflow (scan) → 🟨 (apply)
├─ Phase 3  Carry-patch upstreaming triage + filing            🟦 workflow (audit) → 🟨 (PRs)
├─ Phase 4  Deferred security — deps-r2                         🟩 four-role (gated)
└─ Phase 5  Deferred security — deps-r3 (wasmtime) DECISION     🟨 manual/advisor (ADR-risk gate)

GATED ON M1 MERGE (crates/ mutation — must not race phase-m1-b)
├─ Phase 6  e2e.rs decomposition                               🟦 workflow (plan-the-split) → 🟩 four-role (execute)
├─ Phase 7  Type-state retrofit (6 sites)                      🟩 four-role (gated)
└─ Phase 8  Governance TODO/FIXME sweep + close-out retro      🟦 workflow (audit) → 🟩/🟨 (apply)
```

**Sequencing rule:** Phases 0–5 may proceed immediately in the close-out worktree. Phases 6–8 are *authored* now (plans/briefs drafted) but **blocked** — their impl tasks do not dispatch until `git log origin/governance-v0 ^phase-m1-b` confirms M1 has merged to trunk. The close-out worktree rebases onto post-M1 `governance-v0` before Phase 6 begins.

---

## Phase 0 — Bootstrap the isolated close-out worktree  🟨 manual/advisor

Goal: a dedicated lane so close-out work never shares a working tree or DQ with the live M1 lane (per `multi-lane-worktree.md`).

1. From canonical `brehon-fork`: `git worktree add ../brehon-fork-closeout governance-v0` → branch `phase-v1-closeout` (cut via `bm-cut` brief, or directly for the meta-only early phases).
2. Run the **lane bootstrap checklist** (`feedback_phase_lane_worktree_bootstrap_checklist.md`): `git submodule update --init --recursive`; copy `.mcp.json`, `.env`, `.claude/settings.local.json`; verify the canonical-PMD path guard.
3. Open a new CC session with CWD = `brehon-fork-closeout`. This session drives Phases 1–5. Document **lane mode A** in a bootstrap handover at `.claude/PRPs/handovers/v1-closeout-bootstrap.md`.
4. Confirm no DQ cross-contamination: the close-out lane writes its own `.claude/decision-queue.json`; canonical stays on `governance-v0` meta-edits only.

**Output:** `brehon-fork-closeout/` worktree on `phase-v1-closeout`, bootstrapped, session open.
**User confirm:** none beyond the standard worktree-create (this is a local, non-destructive add).

---

## Phase 1 — Repo & meta hygiene  🟦 workflow (audit) → 🟨 (apply)

Goal: empty the stale-worktree set, archive shipped plans, and right-size the meta-corpus. **No `crates/` touch — zero M1 conflict.**

### 1a. Stale worktree teardown  🟨 manual/advisor (inspect-then-confirm)
Six orphaned dirs on disk, **all with no `.git`** (already-removed worktrees whose dirs survived) and **all empty or near-empty** (verified: 0–1 entries, 8–64 KB; `brehon-fork-scratch/` holds one empty `sl-b/`):
`brehon-fork-deps-r1`, `brehon-fork-phase-v1-AD-a`, `brehon-fork-redaction-r1`, `brehon-fork-scratch`, `brehon-fork-ship-1`, `brehon-fork-ship-2`.

- `git worktree prune` is already clean (git knows they're gone) — so these are pure filesystem cleanup, NOT `git worktree remove`.
- **Per `no-destructive-defaults.md`:** re-inspect each (`ls -A`) at run time, confirm still empty, then `rm -rf` **one at a time with the dir name echoed**. This is the one destructive step — it requires explicit user go-ahead listing the six paths.
- Also reconcile any *registered-but-stale* worktrees the roadmap notes (`brehon-fork-rt-r3/r4/r5`, `quality-r1`, `fed-in-d`) — these may still be registered; use `git worktree remove` (NOT `rm -rf`) for those, `--force` only if a submodule worktree blocks (`feedback_worktree_remove_force_for_submodules.md`).

### 1b. Plan-file archival  🟦 workflow → 🟨 apply
54 active plan files in `.claude/PRPs/plans/`; only 7 in `completed/`. Most active files are for `done` sub-phases.
- **Workflow:** fan out a classifier — one agent per plan file (or batched) cross-references the filename against `v1-roadmap.json` lane status and returns `{file, sub_phase, status, safe_to_archive}`. A completeness-critic agent verifies no plan cited by an *active* rule/brief is moved (grep each candidate's basename across `.claude/{rules,commands,PRPs/briefs,lessons}`). Output: a structured archive manifest.
- **Apply (🟨):** `git mv` the confirmed-done plans into `.claude/PRPs/plans/completed/`; commit `chore(plans): archive N shipped sub-phase plans`. M1 plans (`m1*.plan.md`) stay active.

### 1c. MEMORY.md milestone-prune  🟨 manual/advisor (skill)
MEMORY.md is over budget (202 lines, limit 200; SessionStart WARN fires). A v1→M1 milestone boundary is the highest-yield prune trigger.
- Invoke the **`memory-prune`** skill (Step 2.5 milestone sweep). Surfaces findings → user confirms → applies. Retires prior-milestone phase-texture (the archived-2026-05-31 class, shipped-fix cards) in one pass.

### 1d. Lesson-corpus sweep (optional, advisory)  🟦 workflow
194 lessons in `.claude/lessons/`. Not pruning code lessons (they're load-bearing), but a workflow can surface **superseded/duplicate** pairs (e.g. the historically-cited-but-never-authored slugs the rules already note) for the user to decide. Read-only audit → report; no auto-delete.

**SEED — broken citations in ALWAYS-LOAD rules (verified 2026-06-04, via the `new-lane` skill's Step-7 citation-integrity sweep).** This is the concrete starting point for the lesson-corpus audit — distinct from dedup. The sweep (`grep -rhoE '(feedback|reference)_[a-z0-9_]+\.md' .claude/rules/ | sort -u | while read f; do [ -f ".claude/lessons/$f" ] || echo "$f"; done`) found **3 always-load-rule citations resolving to files that do NOT exist in `.claude/lessons/`** — every session inherits these broken pointers (the GAP-2 class from `v1-closeout-lane-setup-report.md`, now generalised). All three are cited by the always-load rule `.claude/rules/advisor-orchestrator.md`:

| Missing lesson | Cited at (always-load) | Status / action |
|---|---|---|
| `feedback_advisor_watchpoint_specificity.md` | `advisor-orchestrator.md` §3.5 (watchpoint-specificity gate — load-bearing plan-approval gate) | **Author it** — the gate's behaviour is documented inline in §3.5 + indexed in MEMORY.md; reconstruct the lesson from those + git history of watchpoint DQs. |
| `feedback_verify_automated_reviewer_claims_against_compiler.md` | `advisor-orchestrator.md` §5.4 (falsifiable-hypothesis CR-finding variant — compile-check before triaging a CR trait/type claim) | **Author it** — content sketched inline in §5.4 + the PR#132 `.get(0)`→`.first()` Diesel incident; reconstruct from there. |
| `feedback_daemon_telegram_completion_hook.md` | `advisor-orchestrator.md` §1 (Telegram completion hook check) | **Likely already resolved** — reconstructed as a draft this session (GAP 2 fix in the close-out worktree); confirm it's committed to `governance-v0`, else author from the draft. |

Remediation is **authoring 2–3 short lessons** (≤ TIER-2 risk, `.claude/lessons/` only — meta-work, direct-commit per `phase-branch.md`), NOT a workflow fan-out — but the Phase-1d workflow should *also* re-run the Step-7 sweep at completion to confirm zero TIER-1 breaks remain, and surface the ~14 TIER-2 (handover) breaks as a separate lower-priority list. The `new-lane` skill Step 7 splits TIER-1/TIER-2 and carries this ~3+14 baseline so future lanes flag only *new* drift.

**Outputs:** empty stale-worktree set; shipped plans archived; MEMORY.md under budget; (optional) lesson-dedup report; **3 TIER-1 broken always-load-rule citations resolved (lessons authored); Step-7 sweep re-run shows 0 TIER-1 breaks.**

---

## Phase 2 — Documentation-drift reconcile  🟦 workflow (scan) → 🟨 (apply)

Goal: flip the known doc-drift the roadmap itself flags, so the design docs match the shipped code. **No `crates/` logic change** (registry/comment edits only).

Known drift (from `v1-roadmap.json` → `what_remains.documentation_drift_followups` + lane `known_followups`):
- **SL ENTRY_KIND markers** still say `(pending)` in `.claude/rules/governance-log-entry-kind-registry.md` for `SPONSOR_LIABILITY_PENDING/FIRED/ESCAPED` + `ENDORSEMENT_REVOKED` — code emits them correctly; flip to `(active)` mirroring JM-e's line-140 pattern.
- **RT-r2 retro path** in the roadmap points at the deps-r1 retro (placeholder) — correct or annotate.
- **04-data-model-and-api.md** (the LIVING schema doc) — verify it reflects all merged v1 schema (it's code-derived; a workflow can diff doc claims vs `schema.rs`).

- **Workflow:** fan out doc-vs-code verifiers — each takes one drift claim, greps the code for ground truth, returns `{claim, doc_says, code_says, fix}`. Adversarial verify (a second agent compile/grep-checks each proposed fix before it's trusted — per `feedback_verify_automated_reviewer_claims_against_compiler.md`).
- **Apply (🟨):** registry/doc edits are `.claude/` + `docs/` only → direct-commit on `phase-v1-closeout` (or canonical `governance-v0`) per `phase-branch.md` "direct" set. `docs(registry):` / `docs(rules):` subjects.

**Output:** registry markers flipped; roadmap placeholders corrected; LIVING schema doc verified current.

---

## Phase 3 — Carry-patch upstreaming triage + filing  🟦 workflow (audit) → 🟨 (PRs)

Goal: contribute the generic, non-governance carry-patches to upstream LemmyNet, then track their PR numbers in-tree per the existing convention (`feedback_carry_patch_todos.md`). **Analysis is read-only; PR filing is user-driven; the only in-tree edit is filling `#___` placeholders — deferred until M1 merges since 2 candidates live in `crates/`.**

Audit complete (this planning pass): 11 `TODO(brehon-fork)` markers → **4 upstreamable**, **7 fork-local-forever**.

| Candidate | File | What it does | Verdict |
|---|---|---|---|
| Windows signal handling ×2 | `crates/server/src/lib.rs:55,284` | `#[cfg(not(windows))]`-gates `tokio::signal::unix`; ctrl-c fallback on Windows | ✅ upstreamable (generic portability) |
| clippy `#[expect]` removal | `crates/diesel_utils/src/pagination.rs:236` | drops a no-longer-firing `multiple_bound_locations` expectation under `-D warnings` | ✅ upstreamable (toolchain drift) |
| clippy `#[expect]` removal | `crates/db_views/vote/src/impls.rs:130` | same `#[expect]` drift on `paginate_vote_response` | ✅ upstreamable (toolchain drift) |
| `membership_state` ×7 | `db_schema/{lib,source/person,impls/person}.rs`, `api_crud/user/create.rs`, `db_views/registration_applications/impls.rs` | adds/threads the governance `MembershipState` column | ❌ fork-local (governance-coupled) |

- **Workflow (audit refinement, optional):** the 4 upstreamable candidates can be re-verified in parallel — one agent per candidate confirms it's *genuinely* governance-free (no `membership_state` / governance-type leakage in the diff), drafts an upstream-PR description, and checks whether upstream Lemmy `main` has already fixed it. Output: 2 ready-to-file PR drafts (the 2 Windows signal hunks naturally bundle into one PR; the 2 clippy `#[expect]` removals into a second).
- **File PRs (🟨 user):** the user files the upstream PRs (against `LemmyNet/lemmy`, NOT the fork) using the drafts. I cannot push to upstream — I prepare the branches/diffs and PR bodies; the user submits.
- **Backfill `#___` (deferred behind M1):** once a PR number exists, fill it into the marker. The 2 Windows-signal markers are in `crates/server/src/lib.rs` and the 2 clippy ones in `crates/` — these are `crates/` edits → **wait for M1 merge** (Phase 6+ window) before committing the placeholder fills. The 7 fork-local markers get their TODO text updated to "fork-local (governance) — do not upstream" to stop re-triaging them every rebase (also a `crates/` edit, same deferral).

**Output now:** 2 upstream-PR drafts + branches ready for the user to file. **Output post-M1:** `#___` placeholders filled; fork-local markers annotated.

---

## Phase 4 — Deferred security: deps-r2  🟩 four-role (gated) — ⛔ BLOCKED: NO-ELITEDESK while M1 active

> **NO-ELITEDESK directive (user, 2026-06-04 — `project_closeout_no_elitedesk_m1_active.md`):** while the close-out lane runs, the EliteDesk daemon (`junior@brehon-fork`) is **owned by M1** (`phase-m1-b`) — **zero close-out Junior dispatch**. Phase 4 as specced is 🟩 four-role (Junior `bm-cut`/planning/impl-task/ci-watcher on the daemon) → **that is blocked.** Read-only daemon queries (`list_tasks`, `daemon_status`) are still fine. Holds until `phase-m1-b` merges to `governance-v0`.
>
> **When Phase 4 comes up, SURFACE to user — do NOT auto-pick** (per the directive):
> - **(a) Defer Phase 4 entirely** until M1 ships and the daemon frees (cleanest; security fixes land later). Phase 4 then runs normal four-role.
> - **(b) Run deps-r2 laptop-local** — advisor/subagent writes the Rust, laptop cargo+e2e validates (per `project_laptop_canonical_cargo_runner.md`), CR review on the PR, NO Junior dispatch. Honors no-EliteDesk but loses the four-role gating granularity.
>
> Either way, T2's `[patch.crates-io]` Cargo.toml edit also stays gated behind M1 merge (overlap risk, per the approval decision). T3 (Dependabot dispute, GitHub UI) and T4 (wasmtime log DQ) are laptop-local and unaffected by the daemon block — they can run anytime.

Goal: retire 6 of 18 open Dependabot alerts. Precondition `v1-ship-complete` is satisfied (v1 shipped). Fully scoped greenfield brief exists: `.claude/PRPs/briefs/v1-deps-r2-planning-1.md`. **This is a `crates/`-touching, cargo-gated lane** (needs CR review + merge confirm; touches `Cargo.toml`/lockfile/`crates/api/api_utils/src/utils.rs`). Originally specced 🟩 four-role, but the NO-ELITEDESK directive above blocks daemon dispatch — see the two options.

Four tasks (from the brief):
- **T1** — inline webmention replacement (~70 LOC reqwest impl in `crates/api/api_utils/src/utils.rs`), drops the rustls-webpki 0.101.7 chain → closes 3 alerts (1 High).
- **T2** — fork `rlidwka/mdurl.rs` → `barrie-cork/mdurl`, bump idna 0.3→1.1, add the first `[patch.crates-io]` entry to root `Cargo.toml` → closes 1 medium.
- **T3** — file a "dispute advisory" on `astral-tokio-tar 0.6.0` (it IS the fix release) → closes 2 via Dependabot dismissal (🟨 user action in the GitHub UI).
- **T4** — record the wasmtime deferral as a `kind: "log"` DQ entry + dismiss all 13 wasmtime alerts with documented reasons → hands off to Phase 5.

- **M1-overlap check before dispatch:** T1 touches `api_utils/src/utils.rs`; M1 touches `api_utils/src/{notify,plugins,bridge_notify,lib}.rs` — *different files in the same crate*. Low collision risk, but **confirm via `git diff` of M1's actual touch-set at dispatch time** and prefer to sequence Phase 4 *after* M1 merges if any doubt. The `[patch.crates-io]` addition to root `Cargo.toml` is the riskier overlap (M1 may edit Cargo.toml) → safest to gate T2 behind M1.
- **Watchpoints (from brief):** re-enumerate webmention callsites (`rg "webmention::|send_webmention" crates/` must = 6); re-verify the 18-alert total at lane-cut (Dependabot may have re-counted).
- **Pipeline (option a, post-M1 four-role):** `bm-cut` → planning (brief already exists; clarify pass) → impl cohort (T1, T2 serial — both edit build config) → laptop cargo+e2e validation → CR triage → merge confirm.
- **Pipeline (option b, laptop-local, no daemon):** advisor/subagent writes T1 Rust on `phase-v1-closeout` → laptop `cargo-check.bat`/`cargo-test.bat` validate → open PR → CR review → merge confirm. No `bm-cut`/Junior; T2 still gated behind M1.

**Output:** 6 Dependabot alerts retired; `[patch.crates-io]` established; wasmtime deferral logged.

---

## Phase 5 — Deferred security: deps-r3 (wasmtime) DECISION  🟨 manual/advisor (ADR-risk gate)

Goal: decide the wasmtime path. **No brief exists; high risk against ADR-012 (Extism plugin host).** This phase is a **user decision point, not an execution lane** — the 13 wasmtime alerts are aarch64-only criticals and Brehon deploys x86_64, so the real-world exposure is low and the right call may be "keep deferred."

The situation (verified): extism 1.21.0 pins wasmtime 41.0.4 via internal wiggle-macro APIs, so a bare `[patch.crates-io]` override won't compile — closing these alerts needs **forking extism** (1–3 days, high ADR-012 risk) OR adopting an upstream extism release (tracking `extism/extism` PR #847, open since 2026-04-07).

**Surface to user — three options:**
1. **Keep deferred** (recommended default) — document the x86_64-not-vulnerable rationale, suppress all 13 in Dependabot, revisit when upstream extism ships wasmtime ≥42. (Already the roadmap's `deferred_until` posture.)
2. **Fork-and-patch now** — author a deps-r3 brief, fork extism, four-role pipeline it. Only if the user wants a fully-green security tab before pilot.
3. **Wait-and-watch** — set a `/schedule` check on extism PR #847 so we adopt the upstream release the moment it lands.

This phase produces a **decision + (if option 1/3) documented suppressions**, not code, unless the user picks option 2.

---

## Phase 6 — e2e.rs decomposition  🟦 workflow (plan-the-split) → 🟩 four-role (execute) — **GATED ON M1 MERGE**

Goal: break the 18,526-line `crates/server/tests/e2e.rs` monolith into per-domain modules. **This is the single biggest code-side refactor and the one with the hardest M1 conflict** — M1 actively edits `e2e.rs` (adds `m1_*_fixtures` modules). **Does not start until M1 has merged to `governance-v0` and the close-out worktree has rebased onto it.**

Why it matters: the monolith is a documented, recurring Junior Edit-hang hazard (`feedback_junior_worker_e2e_edit_hang.md`, `feedback_fix_impl_pre_locate_e2e_anchors.md`) — every phase that edits it pays an anchor-collision + hang tax. Decomposing it removes that tax for all future milestones (M2, M3).

- **Workflow (plan-the-split, read-only):** fan out analyzers — each maps one logical test domain (jury, sponsor-liability, appeal, reputation, federation-inbound, admin, redaction, ship, m1-messaging) to its line ranges and its shared-fixture dependencies. A synthesis agent proposes a module boundary set (`tests/e2e/{jury,sponsor,appeal,…}.rs` + a shared `common/fixtures.rs`) and flags every cross-domain helper that must move to `common`. Output: a precise, line-ranged split manifest. **This is exactly the fan-out-then-synthesize shape workflows excel at**, and it keeps the 18K-line file out of the main context.
- **Execute (🟩 four-role):** the split itself is a careful, compile-gated, test-must-still-pass refactor → four-role pipeline. Critical guardrail: **behaviour-preserving only** — same tests, same assertions, just relocated; the full e2e suite must show identical pass-count before/after (`feedback_emit_added_requires_full_e2e_gate.md`). Likely several sub-phases (one module-group per sub-phase) to keep each diff reviewable and each cargo run green.
- **Risk:** Rust test-module visibility + shared-fixture re-exports (`feedback_rust_visibility_cross_crate.md`); `mod` declarations and `#[path]` if needed. The workflow's split manifest de-risks this by enumerating every move up front.

**Output:** `e2e.rs` decomposed into per-domain modules; identical test coverage; future e2e edits no longer hang Junior.

---

## Phase 7 — Type-state retrofit (6 sites)  🟩 four-role (gated) — **GATED ON M1 MERGE**

Goal: replace ad-hoc `case.status` match-guards with the `GovernanceCase<S>` phantom type-state wrapper at the 6 annotated sites. **Deferred behind M1** — the sites live in `crates/api/api/src/governance/` alongside M1's `mod.rs`/`admin_config.rs`/`messaging_config.rs` edits; sharing the directory + `mod.rs` risks merge conflict.

The 6 `TODO(type-state)` sites (canonical pattern: `feedback_governance_type_state_handlers.md` — `GovernanceCase<S>` + `TryFrom<ModerationCase>` centralising the `CaseStatus` guard):
- `accept_jury_assignment.rs:110` — dual-role match → `GovernanceCase<JurySelection>`
- `admin_assign_jury.rs:122` → `GovernanceCase<PreJuryAssignable>`
- `admin_close_case.rs:65` — inverted guard (reject Closed only)
- `admin_trigger_appeal_rejury.rs:68` — single-variant guard (cleanest candidate)
- `sponsor_liability_grace.rs:135` — filter-query pattern → exhaustive match
- `submit_jury_vote.rs:271` — terminal-state idempotency guard → `GovernanceCase<Active>`

- **Four-role:** each site is a focused, compile-gated handler change with e2e coverage → plan → impl cohort (sites are independent → `[P]` parallel-eligible, but they share `governance/mod.rs` for the wrapper definition, so Task 0 defines the type-state scaffold non-`[P]`, then the 6 retrofits fan out). CR + merge gates apply.
- **Not a workflow:** these are governance-semantic changes to shipped code with real behaviour-equivalence stakes — four-role gating, not background fan-out.

**Output:** 6 type-state TODOs resolved; `CaseStatus` guards centralised; the type-state pattern's retrofit debt cleared.

---

## Phase 8 — Governance TODO/FIXME sweep + close-out retro  🟦 workflow (audit) → 🟩/🟨 (apply) — **GATED ON M1 MERGE**

Goal: triage the remaining ~38 governance `TODO/FIXME/XXX/HACK` markers (the non-type-state, non-carry-patch residue) and write the v1 lane-meta retro.

- **Workflow (audit):** fan out — one agent per marker cluster reads the TODO + context and classifies `{marker, file, category(bug|defer-to-v2|stale|actionable), recommended_action}`. A completeness-critic confirms none is a latent correctness bug hiding as a TODO (silent-failure lens). Output: a triaged TODO ledger.
- **Apply:** actionable + low-risk → 🟩 four-role fix cohort (or 🟨 direct for comment-only). Defer-to-v2 → annotate + log. Stale → delete.
- **Close-out retro (🟨):** author the v1 capstone meta-retro (`.claude/PRPs/reports/v1-closeout-retro.md`) — the four-role retro signals across the whole v1 arc, the `retro-bypass.jsonl` rate trend, and what carries into M1+. Gate 6 (retro sign-off) → optional `/brehon-phase-transition` to mark v1 formally closed.

**Output:** governance TODO debt triaged + retired; v1 formally closed with a capstone retro.

---

## Cross-cutting guardrails (apply to every phase)

- **M1-isolation invariant (the prime directive):** no Phase touches a file in M1's implement-set (`e2e.rs`, `governance/{mod,messaging_config,admin_config}.rs`, `db_schema/{newtypes,schema, …/governance/…}`, `api_utils/{notify,plugins,bridge_notify,lib}.rs`, `Cargo.toml`) until `git log origin/governance-v0 ^phase-m1-b` is empty (M1 merged). Phases 0–5 are structured to respect this *by construction* (they touch `.claude/`, `docs/`, filesystem, and non-overlapping `crates/` only — with the T1/T2 caveat flagged in Phase 4).
- **NO-ELITEDESK invariant (2026-06-04 user directive — `project_closeout_no_elitedesk_m1_active.md`):** while close-out runs, M1 owns the EliteDesk daemon — **zero close-out Junior dispatch** (no `bm-cut`/planning/impl-task/ci-watcher tasks on `junior@brehon-fork`). Close-out is **laptop-local only**: 🟦 workflow phases run via the laptop `Workflow` tool (subagents in the laptop harness, not the daemon); 🟨 manual/advisor phases are laptop-local; **Phase 4 (the one 🟩 four-role phase) is BLOCKED → surface options to user** (defer vs laptop-local impl). Read-only daemon queries (`list_tasks`, `daemon_status`, `list_hooks`) are fine. Holds until `phase-m1-b` merges. Reinforces (does not replace) the laptop-canonical-cargo-runner rule.
- **Lane discipline:** all close-out work in `brehon-fork-closeout`; its own DQ; canonical `brehon-fork` does meta-edits only (`multi-lane-worktree.md`). One session, one CWD, one lane.
- **No-destructive-defaults:** the only `rm -rf` is Phase 1a (six verified-empty dirs) — inspect-then-confirm, one dir at a time, with explicit user go-ahead.
- **Workflow opt-in & cost:** user opt-in is given; each 🟦 launch still shows its approval prompt; first run scoped small to gauge burn; user can stop any run.
- **Direct-vs-PR:** meta/docs/registry edits commit direct on the branch per `phase-branch.md`; every `crates/` change goes through the PR flow with CodeRabbit.

---

## Verification — how we'll know each tier is done

**Hygiene tier (Phases 1–2):**
- `ls -d C:/Users/barri/Developer/brehon-fork-*` shows only canonical + `brehon-fork-validate` (M1) + `brehon-fork-closeout` (this lane).
- `ls .claude/PRPs/plans/*.plan.md | wc -l` drops to the active-only set (M1 + close-out); shipped plans in `completed/`.
- `wc -l MEMORY.md` ≤ 200; SessionStart WARN no longer fires.
- `grep "(pending)" .claude/rules/governance-log-entry-kind-registry.md` returns no SL markers that code actually emits.

**Security tier (Phases 4–5):**
- GitHub Dependabot alert count drops by 6 (deps-r2); the remaining 13 wasmtime alerts are either closed (option 2) or documented-suppressed (option 1/3).
- `grep -A2 "patch.crates-io" Cargo.toml` shows the mdurl fork pin (deps-r2 T2).
- `rg "webmention::" crates/` returns 0 external-crate references (T1 inlined); full e2e suite green via laptop cargo.

**Upstreaming tier (Phase 3):**
- 2 upstream PRs filed against `LemmyNet/lemmy` (user-submitted from the prepared drafts).
- Post-M1: `grep "TODO(brehon-fork)" crates/` markers carry either a real PR number or "fork-local — do not upstream"; no bare `#___` on the 4 upstreamed hunks.

**Code-refactor tier (Phases 6–8, post-M1):**
- `wc -l crates/server/tests/e2e.rs` drops dramatically; `ls crates/server/tests/e2e/` shows the per-domain modules; full e2e pass-count identical to pre-split (run via laptop `cargo-test.bat --workspace --test e2e --features full`).
- `grep "TODO(type-state)" crates/` returns 0.
- `grep -ic "TODO\|FIXME" governance code` materially reduced; triage ledger shows each marker resolved/deferred/deleted.
- v1 close-out retro on trunk; `/brehon-phase-transition` (optional) marks v1 closed.

---

## Open decisions — RESOLVED at approval (2026-06-04)

1. **Phase 4 sequencing** — ✅ RUN NOW, gate only T2. T1/T3/T4 proceed in the close-out lane immediately; T2's Cargo.toml `[patch.crates-io]` edit waits for M1 merge.
2. **Phase 5 wasmtime** — ✅ KEEP DEFERRED + document + suppress 13 alerts + `/schedule` watch on `extism/extism` PR #847.
3. **Promotion** — ✅ promoted to `.claude/PRPs/plans/v1-closeout.plan.md` at Phase 0.
4. **First workflow** — ✅ Phase 3 carry-patch audit is the cost-gauge run.

## Handoff model (2026-06-04)

This advisor session (canonical `brehon-fork`) **prepares** the close-out lane and hands off. The **user drives execution** in a separate, fully-harnessed `brehon-fork-closeout` session. Phase 0 deliverables from this session:
- Tracked plan committed to `governance-v0`.
- `brehon-fork-closeout` worktree created on `phase-v1-closeout`, **fully bootstrapped** (submodules + `.mcp.json` + `.env` + `.claude/settings.local.json` so PMD MCP, Junior MCP, and all SessionStart hooks are live — per `feedback_phase_lane_worktree_bootstrap_checklist.md` + `pmd-invariants.md` #1/#5).
- Cold-start bootstrap handover at `.claude/PRPs/handovers/v1-closeout-bootstrap.md` (committed) — the user's next session reads this with zero conversation context.
- **Isolation guarantee:** the close-out lane never shares a working tree, branch, or DQ with the live M1 lane (`brehon-fork-validate` / `phase-m1-b`). PMD is intentionally shared (HTTP daemon, cross-lane by design — not a race surface).
